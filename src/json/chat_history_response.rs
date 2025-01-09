use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatHistoryResponse {
    pub code: i64,
    pub msg: String,
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    pub history: Vec<History>,
    pub page: i64,
    pub total: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct History {
    pub chat_id: String,
    pub title: String,
    pub role_name: String,
}

