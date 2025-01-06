use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};

// 定义会话开始的请求结构
#[derive(Debug, Deserialize)]
struct StartSessionPayload {
    session_id: String,
    input_format: String,
    output_format: String,
    sample_rate: u32,
    output_sample_rate: u32,
    round: u32,
}

#[derive(Debug, Deserialize)]
struct WebSocketMessage {
    #[serde(rename = "type")]
    msg_type: String,
    payload: Value,
}

// WebSocket upgrade handler
pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

// 处理 WebSocket 连接
async fn handle_socket(mut socket: WebSocket) {
    let mut session_started = false;

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to receive message: {}", e);
                break;
            }
        };

        // 处理接收到的消息
        if let Message::Text(text) = msg {
            let ws_msg: WebSocketMessage = match serde_json::from_str(&text) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to parse message: {}", e);
                    continue;
                }
            };

            match ws_msg.msg_type.as_str() {
                "start_session" => {
                    if let Ok(payload) = serde_json::from_value::<StartSessionPayload>(ws_msg.payload) {
                        info!("Starting session: {:?}", payload);
                        session_started = true;
                        let started = json!({
                            "type": "session_started"
                        });
                        
                        // 发送会话开始确认
                        if let Err(e) = socket.send(Message::Text(started.to_string().into())).await {
                            error!("Failed to send session_started: {}", e);
                            break;
                        }
                    }
                }
                "audio_input_chunk" => {
                    if !session_started {
                        error!("Received audio chunk before session start");
                        continue;
                    }

                    // 处理音频数据
                    if let Value::String(audio_data) = ws_msg.payload {
                        // 这里可以添加音频处理逻辑
                        // 示例：简单地回显相同的音频数据
                        if let Err(e) = socket.send(Message::Text(json!({
                            "type": "audio_output_chunk",
                            "payload": audio_data
                        }).to_string().into())).await {
                            error!("Failed to send audio output: {}", e);
                            break;
                        }
                    }
                }
                "audio_input_finish" => {
                    // 发送处理完成信号
                    if let Err(e) = socket.send(Message::Text(json!({
                        "type": "audio_output_finished"
                    }).to_string().into())).await {
                        error!("Failed to send finish signal: {}", e);
                    }
                    // 等待5秒后关闭连接
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    break;
                }
                _ => {
                    error!("Unknown message type: {}", ws_msg.msg_type);
                }
            }
        }
    }

    // 连接关闭
    info!("WebSocket connection closed");
}
