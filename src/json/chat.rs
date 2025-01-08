use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: String,
    pub user_id: String,
    pub role_id: String,
}

pub struct ChatResponse {
    pub message: String,
}