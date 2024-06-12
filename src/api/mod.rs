pub mod errors;
pub mod utils;

use self::errors::ServerError;
use crate::Bindings;
use crate::models::{Chat, CreateChat, Message, PostMessage, SetTitle};
use crate::models::openai::Role;
use ohkami::typed::{status, DataStream};
use ohkami::serde::Deserialize;
use web_sys::{js_sys, wasm_bindgen::JsCast, WorkerGlobalScope};


#[worker::send]
pub async fn list_chats(
    b: Bindings,
) -> Result<Vec<Chat>, ServerError> {
    let chats = b.DB
        .prepare("SELECT id, title FROM chats")
        .all().await?.results()?;

    Ok(chats)
}

#[worker::send]
pub async fn create_chat(
    b: Bindings,
    req: CreateChat,
) -> Result<status::Created<Chat>, ServerError> {
    let id = WorkerGlobalScope::unchecked_from_js(js_sys::global().into())
        .crypto().unwrap().random_uuid();

    b.DB.batch([
        Some(b.DB
            .prepare("INSERT INTO chats (id, title) VALUES (?1, ?2)")
            .bind(&[(&id).into(), req.title.as_deref().into()])?
        ),
        req.system_instruction.is_some().then_some(b.DB
            .prepare("INSERT INTO messages (chat_id, role_id, content) VALUES (?1, ?2, ?3)")
            .bind(&[
                (&id).into(),
                (Role::system.id()).into(),
                req.system_instruction.unwrap().into()
            ])?
        )
    ].into_iter().flatten().collect()).await?;

    Ok(status::Created(Chat { id, title: req.title }))
}

#[worker::send]
pub async fn load_messages(chat_id: &str,
    b: Bindings,
) -> Result<Vec<Message>, ServerError> {
    let records = {
        #[derive(Deserialize)]
        struct MessageRecord { id: usize, role_id: u8, content: String }

        b.DB.prepare("SELECT id, role_id, content FROM messages WHERE chat_id = ?")
            .bind(&[chat_id.into()])?
            .all().await?.results::<MessageRecord>()?
    };

    Ok(records.into_iter().map(|r| Message {
        id:      r.id,
        role:    Role::from_id(r.role_id),
        content: r.content,
    }).collect())
}

#[worker::send]
pub async fn post_message(chat_id: &str,
    b: Bindings,
    req: PostMessage,
) -> Result<DataStream<String>, ServerError> {
    todo!()
}

#[worker::send]
pub async fn set_title(chat_id: &str,
    b: Bindings,
    req: SetTitle,
) -> Result<(), ServerError> {
    

    todo!()
}

#[worker::send]
pub async fn update_message(message_id: &str,
    b: Bindings,
    req: PostMessage,
) -> Result<DataStream<String>, ServerError> {
    todo!()
}
