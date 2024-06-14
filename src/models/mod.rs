pub mod openai;

use ohkami::typed::{Payload, Query};
use ohkami::builtin::payload::{JSON, Text};


#[derive(ohkami::serde::Deserialize, Debug)]
pub struct IDObject {
    pub id: usize,
}

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

#[Query]
pub struct LoadMessagesQuery {
    pub branch: usize,
}

#[Payload(JSON/SD)]
pub struct PostMessage {
    pub content: String,
}

#[Payload(Text/SD)]
pub struct SetTitle(pub String);

pub struct BranchID([u8; 6]);
const _: () = {
    impl BranchID {
        pub fn new() -> Self {
            use web_sys::{js_sys, wasm_bindgen::JsCast, WorkerGlobalScope};
            
            let mut bytes = <[u8; 6]>::default();
            WorkerGlobalScope::unchecked_from_js(js_sys::global().into())
                .crypto().unwrap()
                .get_random_values_with_u8_array(&mut bytes).unwrap();
        
            Self(bytes)
        }
    }

    impl std::ops::Deref for BranchID {
        type Target = str;
        fn deref(&self) -> &Self::Target {
            /* SAFETY: UUID consists of asciis */
            unsafe {std::str::from_utf8_unchecked(&self.0)}
        }
    }
};

#[derive(ohkami::serde::Serialize)]
pub struct ResponseChunk {
    pub id:         usize,
    pub message_id: usize,
    pub diff:       String,
    pub finish_by:  Option<openai::ChatCompletionFinishReason>,
}
const _: () = {
    impl Into<String> for ResponseChunk {
        #[inline(always)]
        fn into(self) -> String {
            unsafe {/* see serde_json::to_string */
                String::from_utf8_unchecked(
                    <JSON as ohkami::typed::PayloadType>::bytes(&self).unwrap()
                )
            }
        }
    }
};
