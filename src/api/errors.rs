use ohkami::prelude::*;


#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Worker error: {0}")]
    Worker(#[from] worker::Error),

    #[error("Reqwest fetch failed: {0}")]
    Fetch(#[from] reqwest::Error),

    #[error("Failed to deserialize: {msg}")]
    Deserialize { msg: String },

    #[error("Failed to serialize: {msg}")]
    Serialize { msg: String },
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        worker::console_error!("{self}");

        match self {
            Self::Worker(_)       => Response::InternalServerError(),
            Self::Fetch(_)        => Response::InternalServerError(),
            Self::Deserialize{..} => Response::InternalServerError(),
            Self::Serialize{..}   => Response::InternalServerError(),
        }
    }
}

#[cfg(debug_assertions)]
fn __() {
    fn assert_send<T: Send>() {}
    assert_send::<ServerError>()
}
