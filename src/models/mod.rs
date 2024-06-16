pub mod openai;

use ohkami::serde::{Serialize, Deserialize};
use ohkami::typed::{Payload, Query};
use ohkami::builtin::payload::{JSON, Text};


#[derive(Deserialize, Debug)]
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

#[Payload(JSON/SD)]
pub struct CreateChat {
    pub title:              Option<String>,
    pub system_instruction: Option<String>,
}

#[Query]
pub struct LoadMessagesQuery {
    pub branch: BranchID,
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
        #[inline]
        pub fn new() -> Self {
            use web_sys::{js_sys, wasm_bindgen::JsCast, WorkerGlobalScope};
            
            let mut bytes = <[u8; 6]>::default();
            WorkerGlobalScope::unchecked_from_js(js_sys::global().into())
                .crypto().unwrap()
                .get_random_values_with_u8_array(&mut bytes).unwrap();        
            Self(bytes)
        }
        #[inline]
        pub const fn as_str(&self) -> &str {
            unsafe {std::str::from_utf8_unchecked(&self.0)}
        }
    }
    impl From<&str> for BranchID {
        #[inline]
        fn from(s: &str) -> Self {
            ohkami::FromParam::from_param(s.into()).unwrap()
        }
    }
};
impl<'de> Deserialize<'de> for BranchID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: ohkami::serde::Deserializer<'de> {
        let str = <&'de str>::deserialize(deserializer)?;
        ohkami::FromParam::from_param(str.into())
            .map_err(|e: ohkami::Response| ohkami::serde::de::Error::custom(
                String::from_utf8_lossy(e.payload().unwrap())
            ))
    }
}
impl<'req> ohkami::FromParam<'req> for BranchID {
    type Error = ohkami::Response;
    #[inline]
    fn from_param(param: std::borrow::Cow<'req, str>) -> Result<Self, Self::Error> {
        (param.len() == 6)
            .then(|| Self(param.as_bytes().try_into().unwrap()))
            .ok_or_else(|| ohkami::Response::BadRequest().with_text(format!("Inlvaid branch id: `{param}`")))
    }
}

#[derive(Serialize)]
pub struct ResponseChunk {
    pub message_id:  usize,
    pub response_id: usize,
    pub diff:        String,
    pub finish_by:   Option<openai::ChatCompletionFinishReason>,
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
