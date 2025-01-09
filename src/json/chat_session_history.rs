use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatSessionHistoryResponse {
    pub code: i64,
    pub msg: String,
    pub history: Vec<History>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct History {
    pub id: String,
    pub user: String,
    pub assistant: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ChatSessionHistoryRequest {
    pub offset: i64,
    pub limit: i64,
    pub chat_id: String,
}
