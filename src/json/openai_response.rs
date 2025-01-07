use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: i64,
    pub model: String,
    pub service_tier: Option<serde_json::Value>,
    pub system_fingerprint: String,
    pub object: String,
    pub usage: Usage,
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    pub index: i64,
    pub message: Message,
    pub finish_reason: String,
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub refusal: Option<serde_json::Value>,
    pub tool_calls: Option<serde_json::Value>,
    pub role: String,
    pub function_call: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub prompt_tokens_details: Option<serde_json::Value>,
    pub completion_tokens_details: Option<serde_json::Value>,
}
