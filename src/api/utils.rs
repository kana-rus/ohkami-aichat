use crate::models::openai;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};


#[derive(Clone)]
pub struct ChatResponseBuffer(
    Arc<Mutex<ChatResponseIncomplete>>
);
struct ChatResponseIncomplete {
    text:      String,
    finish_by: Option<openai::ChatCompletionFinishReason>,
}

impl ChatResponseBuffer {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(ChatResponseIncomplete {
            text:      String::new(),
            finish_by: None,
        })))
    }

    pub fn push(&self, diff: &str) {
        self.0.lock().unwrap().text.push_str(diff)
    }
    pub fn finish(&self, reason: openai::ChatCompletionFinishReason) {
        self.0.lock().unwrap().finish_by = Some(reason)
    }

    pub async fn complete(self) -> (String, openai::ChatCompletionFinishReason) {
        struct ChatResponseComplete(ChatResponseBuffer);
        impl std::future::Future for ChatResponseComplete {
            type Output = (String, openai::ChatCompletionFinishReason);
            fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let this = &mut self.0.0.lock().unwrap();
                if this.finish_by.is_some() {
                    Poll::Ready((std::mem::take(&mut this.text), this.finish_by.unwrap()))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }

        ChatResponseComplete(self).await
    }
}
