use ohkami::{typed::Payload, builtin::payload::JSON};
use ohkami::serde::{Deserialize, Serialize};


#[Payload(JSON/SD)]
pub struct ChatCompletions {
    pub model:    &'static str,
    pub messages: Vec<ChatMessage>,
    pub stream:   bool,
}
#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub role:    Role,
    pub content: String,
}

#[Payload(JSON/DS)]
pub struct ChatCompletionChunk {
    pub id:      String,
    pub choices: [ChatCompletionChoice; 1],
}
#[derive(Deserialize, Serialize)]
pub struct ChatCompletionChoice {
    pub index:         usize,
    pub delta:         ChatCompletionDelta,
    pub finish_reason: Option<ChatCompletionFinishReason>,
}
#[derive(Deserialize, Serialize)]
pub struct ChatCompletionDelta {
    pub role:    Option<Role>,
    pub content: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub enum ChatCompletionFinishReason {
    stop,
    length,
    content_filter,
}
const _: () = {
    use ohkami::serde::{ser, de};
    use ohkami::typed::PayloadType as _;

    impl ChatCompletionChunk {
        #[inline]
        pub fn from_raw(chunk: &str) -> Result<Self, impl de::Error + '_> {
            JSON::parse(chunk.as_bytes())
        }
        #[inline]
        pub fn into_raw(&self) -> Result<String, impl ser::Error + '_> {
            JSON::bytes(self)
                .and_then(|bytes| String::from_utf8(bytes)
                    .map_err(|e| ser::Error::custom(e.to_string())))
        }
    }
};

#[derive(Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub enum Role {
    system,
    user,
    assistant,
}
const _: () = {
    impl Role {
        pub const fn id(self) -> u8 {
            self as _
        }
        pub const fn from_id(id: u8) -> Self {
            match id {
                0..2 => unsafe {std::mem::transmute(id)},
                _ => panic!("")
            }
        }
    }
};
