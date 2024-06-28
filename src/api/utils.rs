use crate::models::{self, openai};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use ohkami::utils::stream::{self, StreamExt};
use ohkami::serde::json;


#[derive(Clone)]
pub struct ChatResponseBuffer(
    Arc<Mutex<ChatResponseIncomplete>>
);
struct ChatResponseIncomplete {
    text:      String,
    finish_by: Option<openai::ChatCompletionFinishReason>,
}
const _: () = {

    impl ChatResponseBuffer {
        pub fn new() -> Self {
            Self(Arc::new(Mutex::new(ChatResponseIncomplete {
                text:      String::new(),
                finish_by: None,
            })))
        }
        
        pub fn push(&self, chunk: &models::ResponseChunk) {
            let this = &mut self.0.lock().unwrap();
            this.text.push_str(&chunk.diff);
            this.finish_by = chunk.finish_by;
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
};
    
pub struct ChatCompletionStream(Pin<Box<dyn
    stream::Stream<Item = models::ResponseChunk>
>>);
const _: () = {
    impl ChatCompletionStream {
        pub async fn from(res: reqwest::Response) -> Self {
            let mut res = res.bytes_stream();

            Self(Box::pin(stream::queue(|mut q| async move {
                let mut push_chunk_line = |mut line: String| {
                    #[cfg(debug_assertions)] {
                        assert!(line.ends_with("\n\n"))
                    }
                    line.truncate(line.len() - 2);

                    if let Ok(chunk) = json::from_str(&line) {
                        q.push(chunk)
                    }
                };
        
                let mut remaining = String::new();
        
                while let Some(Ok(raw_chunk)) = res.next().await {
                    for line in std::str::from_utf8(&raw_chunk).unwrap()
                        .split_inclusive("\n\n")
                    {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data.ends_with("\n\n") {
                                push_chunk_line(data.to_string())
                            } else {
                                remaining = data.into()
                            }
                        } else {
                            push_chunk_line(std::mem::take(&mut remaining) + line)
                        }
                    }
                }
            })))
        }
    }
};
