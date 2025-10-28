use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputMessage {
    pub role: String,
    pub content: String,
}

#[allow(unused)]
impl InputMessage {
    pub fn system(content: impl ToString) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }

    pub fn user(content: impl ToString) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    #[allow(unused)]
    pub fn assistant(content: impl ToString) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<InputMessage>,
    //pub top_k: usize,
    pub top_p: f32,
    //pub n_keep: usize,
    //pub n_predict: usize,
    //pub cache_prompt: bool,
    //pub stop: Vec<String>,
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub stream: bool,
    pub max_tokens: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: i64,
    pub model: String,
    #[serde(rename = "system_fingerprint")]
    pub system_fingerprint: String,
    pub object: String,
    pub usage: Usage,
    #[serde(rename = "time_info")]
    pub time_info: TimeInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    #[serde(rename = "finish_reason")]
    pub finish_reason: String,
    pub index: i64,
    pub message: Message,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub content: String,
    pub role: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i64,
    #[serde(rename = "completion_tokens")]
    pub completion_tokens: i64,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
    #[serde(rename = "prompt_tokens_details")]
    pub prompt_tokens_details: PromptTokensDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTokensDetails {
    #[serde(rename = "cached_tokens")]
    pub cached_tokens: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeInfo {
    #[serde(rename = "queue_time")]
    pub queue_time: f64,
    #[serde(rename = "prompt_time")]
    pub prompt_time: f64,
    #[serde(rename = "completion_time")]
    pub completion_time: f64,
    #[serde(rename = "total_time")]
    pub total_time: f64,
    pub created: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub content_type: String,
    pub body: String,
}

#[allow(unused)]
impl Content {
    pub fn text(content: impl ToString) -> Self {
        Self {
            content_type: "text/plain".to_string(),
            body: content.to_string(),
        }
    }
}
