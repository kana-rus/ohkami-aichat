use crate::models::{openai, IDObject, LoadMessagesQuery, Message};
use super::ServerError;


impl crate::Bindings {
    #[inline(always)]
    pub const fn repository(&self) -> Repository<'_> {
        Repository(&self.DB)
    }
}

pub struct Repository<'d1>(&'d1 worker::D1Database);

impl Repository<'_> {
    pub async fn load_messages_of(&self,
        chat_id: &str,
        q: Option<LoadMessagesQuery>
    ) -> Result<Vec<Message>, ServerError> {
        let branch = match q {
            Some(query) => query.branch,
            None => {
                let Some(branches) = self.0
                    .prepare("\
                        SELECT branches FROM messages
                        WHERE chat_id = ?1
                        ORDER BY id ASC
                        LIMIT 1
                    ")
                    .bind(&[chat_id.into()])?
                    .first::<String>(Some("branches")).await?
                else {return Ok(vec![])};

                let Some((_, latest_branch)) = branches.rsplit_once(':')
                else {return Ok(vec![])};

                latest_branch.parse().unwrap()
            }
        };

        let records = {
            #[derive(ohkami::serde::Deserialize)]
            struct MessageRecord { id: usize, role_id: u8, content: String }

            self.0
                .prepare("\
                    SELECT id, role_id, content FROM messages
                    WHERE chat_id = ?1 AND instr(branches, ':' || ?2)
                    ORDER BY id ASC
                ")
                .bind(&[
                    chat_id.into(),
                    branch.into()
                ])?
                .all().await?.results::<MessageRecord>()?
        };

        Ok(records.into_iter().map(|r| Message {
            id:      r.id,
            role:    openai::Role::from_id(r.role_id),
            content: r.content,
        }).collect())
    }

    pub async fn insert_message_response_pair(&self,
        chat_id:  &str,
        message:  String,
        response: String,
    ) -> Result<[usize; 2], ServerError> {
        let ids = self.0
            .prepare(
                "INSERT INTO messages (chat_id, role_id, content)
                VALUES (?1, ?2, ?3), (?4, ?5, ?6)
                RETURNING id")
            .bind(&[
                chat_id.into(), openai::Role::user.as_id().into(), message.into(),
                chat_id.into(), openai::Role::assistant.as_id().into(), response.into()
            ])?
            .all().await?.results::<IDObject>()?;

        let mut ids = TryInto::<[IDObject; 2]>::try_into(ids).unwrap()
            .map(|IDObject { id }| id);
        ids.sort();
        
        Ok(ids)
    }
}
