mod repository;
mod errors;
mod utils;

use errors::ServerError;
use crate::Bindings;
use crate::models::{openai, IDObject, Chat, CreateChat, Message, ResponseChunk, PostMessage, SetTitle, LoadMessagesQuery};
use ohkami::typed::{status, DataStream};
use ohkami::utils::StreamExt;
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
                openai::Role::system.as_id().into(),
                req.system_instruction.unwrap().into()
            ])?
        ),
    ].into_iter().flatten().collect()).await?;

    Ok(status::Created(Chat { id, title: req.title }))
}

#[worker::send]
pub async fn load_messages(chat_id: &str,
    b: Bindings,
    q: Option<LoadMessagesQuery>,
) -> Result<Vec<Message>, ServerError> {
    let messages = b.repository().load_messages_of(chat_id, q).await?;
    Ok(messages)
}

#[worker::send]
pub async fn set_title(chat_id: &str,
    b: Bindings,
    req: SetTitle,
) -> Result<(), ServerError> {
    b.DB.prepare("UPDATE chats SET title = ?1 WHERE id = ?2")
        .bind(&[req.0.into(), chat_id.into()])?
        .run().await?;

    Ok(())
}

#[worker::send]
pub async fn post_message(chat_id: &str,
    b: Bindings,
    q: Option<LoadMessagesQuery>,
    req: PostMessage,
    ctx: &worker::Context,
) -> Result<DataStream<ResponseChunk, ServerError>, ServerError> {
    let [message_id, response_id] = b.repository()
        .insert_message_response_pair(chat_id, req.content, String::new()).await?;

    let gpt_response = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(b.OPENAI_API_KEY)
        .json(&openai::ChatCompletions {
            model:    "gpt-4o",
            stream:   true,
            messages: b.repository().load_messages_of(chat_id, q).await?
                .into_iter()
                .map(|m| openai::ChatMessage { role: m.role, content: m.content })
                .collect(),
        })
        .send().await?
        .bytes_stream();

    let response_buffer = utils::ChatResponseBuffer::new();

    let stream = DataStream::from_stream(
        gpt_response.map({
            let response_buffer = response_buffer.clone();

            move |chunk| {
                let [choice] = openai::ChatCompletionChunk::from_raw(&chunk?)
                    .map_err(|e| ServerError::Deserialize { msg: e.to_string() })?
                    .choices;

                let message_chunk = ResponseChunk {
                    message_id,
                    response_id,
                    diff:      choice.delta.content.unwrap_or_else(String::new),
                    finish_by: choice.finish_reason,
                };

                response_buffer.push(&message_chunk);

                Ok(message_chunk)
            }
        })
    );

    ctx.wait_until({
        let response_buffer = response_buffer.clone();

        async move {
            let (content, finish_reason) = response_buffer.complete().await;
            if let Err(err) = b.DB
                .prepare(
                    "UPDATE messages SET content = ?1, finish_reason_id = ?2
                    WHERE id = ?3")
                .bind(&[
                    content.into(),
                    finish_reason.as_id().into(),
                    response_id.into(),
                ]).unwrap()
                .run().await
            {
                worker::console_error!("Failed to save full GPT response: {err}");
            }
        }
    });

    Ok(stream)
}

#[worker::send]
pub async fn create_new_branch(message_id: &str,
    b: Bindings,
    req: PostMessage,
) -> Result<DataStream<String>, ServerError> {
    todo!()
}

#[worker::send]
pub async fn regenerate_response(message_id: &str,
    b: Bindings,
    req: PostMessage,
) -> Result<DataStream<String>, ServerError> {
    todo!()
}
