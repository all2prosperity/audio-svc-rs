use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct OpenAIResponse {
    id: String,
    choices: Vec<Choice>,
    created: i64,
    model: String,
    service_tier: Option<serde_json::Value>,
    system_fingerprint: String,
    object: String,
    usage: Usage,
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    index: i64,
    message: Message,
    finish_reason: String,
    logprobs: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    content: String,
    refusal: Option<serde_json::Value>,
    tool_calls: Option<serde_json::Value>,
    role: String,
    function_call: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Usage {
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
    prompt_tokens_details: Option<serde_json::Value>,
    completion_tokens_details: Option<serde_json::Value>,
}
