use reqwest::Client;
use crate::config::OZ_SERVER_CONFIG;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use crate::json::mqtt::{MqttEvent, Payload, MessagePayload, MqttMessage};
use crate::constant::{MQTT_MSG_SOURCE_USER, MQTT_MSG_SOURCE_DEVICE};


async fn get_auth() -> Result<String, anyhow::Error> {
    let auth = format!("{}:{}", OZ_SERVER_CONFIG.get::<String>("mqtt_api_key")?, OZ_SERVER_CONFIG.get::<String>("mqtt_api_secret")?);
    let auth = STANDARD.encode(auth);
    Ok(auth)
}

pub async fn publish_event(event: String, device_id: String) -> Result<(), anyhow::Error> {
    let client = Client::new();

    let payload = Payload {
        event: event,
    };

    let mqtt_event = MqttEvent {
        topic: format!("device/{}/event", device_id),
        payload: serde_json::to_string(&payload)?,
    };

    let response = client
        .post(format!("{}/api/v5/publish", OZ_SERVER_CONFIG.get::<String>("mqtt_url")?))
        .header("Authorization", format!("Basic {}", get_auth().await?))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&mqtt_event)?)
        .send()
        .await?;

    println!("Response: {}", response.text().await?);

    Ok(())
}

pub async fn publish_message(self_message: String, device_message: String, device_id: String) -> Result<(), anyhow::Error> {

    let payload = MessagePayload {
        source: MQTT_MSG_SOURCE_USER.to_string(),
        content: self_message,
    };

    let device_payload = MessagePayload {
        source: MQTT_MSG_SOURCE_DEVICE.to_string(),
        content: device_message,
    };

    let payload: Vec<MessagePayload> = vec![payload, device_payload];

    let mqtt_message = MqttMessage {
        payload: serde_json::to_string(&payload)?,
        topic: format!("app/{}/chat", device_id),
    };

    let client = Client::new();

    let response = client
        .post(format!("{}/api/v5/publish", OZ_SERVER_CONFIG.get::<String>("mqtt_url")?))
        .header("Authorization", format!("Basic {}", get_auth().await?))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&mqtt_message)?)
        .send()
        .await?;

    println!("Response: {}", response.text().await?);

    Ok(())
}