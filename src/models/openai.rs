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
    pub choices: [ChatCompletionChoice; 1],
}
#[derive(Deserialize, Serialize)]
pub struct ChatCompletionChoice {
    pub delta:         ChatCompletionDelta,
    pub finish_reason: Option<ChatCompletionFinishReason>,
}
#[derive(Deserialize, Serialize)]
pub struct ChatCompletionDelta {
    pub role:    Option<Role>,
    pub content: Option<String>,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum ChatCompletionFinishReason {
    stop,
    length,
    content_filter,
}
const _: () = {
    impl Into<String> for ChatCompletionChunk {
        #[inline]
        fn into(self) -> String {
            ohkami::serde::json::to_string(&self).unwrap()
        }
    }

    impl ChatCompletionFinishReason {
        pub const fn as_id(self) -> u8 {
            self as _
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
        pub const fn as_id(self) -> u8 {
            self as _
        }
        pub const fn from_id(id: u8) -> Self {
            match id {
                0..3 => unsafe {std::mem::transmute(id)},
                _ => panic!("Invalid role id")
            }
        }
    }
};
