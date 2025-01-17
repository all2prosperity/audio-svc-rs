use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct MqttEvent {
    pub topic: String,
    pub payload: String,
}

#[derive(Serialize, Deserialize)]
pub struct Payload {
    pub event: String,
}



#[derive(Serialize, Deserialize)]
pub struct MessagePayload {
    pub source: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct MqttMessage {
    pub payload: String,
    pub topic: String,
}
