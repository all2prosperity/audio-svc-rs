use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: String,
    pub user_id: String,
    pub role_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: String,
    pub session_id: String,
    pub role_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatHistoryRequest {
    pub offset: i64,
    pub limit: i64,
}