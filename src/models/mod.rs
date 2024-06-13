pub mod openai;

use ohkami::typed::Payload;
use ohkami::builtin::payload::{JSON, Text};


#[Payload(JSON/SD)]
pub struct Chat {
    pub id:    String,
    pub title: Option<String>,
}

#[Payload(JSON/SD)]
pub struct Message {
    pub id:      usize,
    pub role:    openai::Role,
    pub content: String,
}
impl Message {
    pub async fn load_all(chat_id: &str, d1: &worker::D1Database) -> Result<Vec<Self>, worker::Error> {
        let records = {
            #[derive(ohkami::serde::Deserialize)]
            struct MessageRecord { id: usize, role_id: u8, content: String }
    
            d1.prepare("SELECT id, role_id, content FROM messages WHERE chat_id = ?")
                .bind(&[chat_id.into()])?
                .all().await?.results::<MessageRecord>()?
        };
    
        Ok(records.into_iter().map(|r| Message {
            id:      r.id,
            role:    openai::Role::from_id(r.role_id),
            content: r.content,
        }).collect())
    }
}

#[Payload(JSON/SD)]
pub struct CreateChat {
    pub title:              Option<String>,
    pub system_instruction: Option<String>,
}

#[Payload(JSON/SD)]
pub struct PostMessage {
    pub content: String,
}

#[Payload(Text/SD)]
pub struct SetTitle(pub String);

#[derive(ohkami::serde::Serialize)]
pub struct MessageChunk {
    pub diff:      String,
    pub finish_by: Option<openai::ChatCompletionFinishReason>,
}
impl Into<String> for MessageChunk {
    #[inline(always)]
    fn into(self) -> String {
        unsafe {// see serde_json::to_string
            String::from_utf8_unchecked(
                <JSON as ohkami::typed::PayloadType>::bytes(&self.diff).unwrap()
            )
        }
    }
}
