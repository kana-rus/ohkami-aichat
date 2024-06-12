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
pub struct SetTitle(String);
