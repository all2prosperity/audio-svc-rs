use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json, State,
    },
    response::Response,
    Extension,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use llm_audio_toolkit::asr::{volc::VolcanoConfig, volc::VolcanoEchoMage, EchoMage};
use llm_audio_toolkit::tts::volc::{VolcConfig as TTSConfig, VolcWsTTS};
use llm_audio_toolkit::tts::SpellCaster;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};

use crate::{
    handlers::chat::Chat,
    structures::{user::CurrentUser, AppState},
};

const START_SESSION_MSG: &str = "start_session";
const AUDIO_INPUT_CHUNK_MSG: &str = "audio_input_chunk";
const AUDIO_INPUT_FINISH_MSG: &str = "audio_input_finish";

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
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(mut app_state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, app_state, user))
}

// 处理 WebSocket 连接
async fn handle_socket(mut socket: WebSocket, mut app_state: AppState, user: CurrentUser) {
    let mut session_started = false;
    let mut asr: Option<VolcanoEchoMage> = None;
    let mut role_id: Option<String> = None;
    //let mut audio_buffer = Vec::new();

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to receive message: {}", e);
                break;
            }
        };

        if let Message::Text(text) = msg {
            let ws_msg: WebSocketMessage = match serde_json::from_str(&text) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to parse message: {}", e);
                    continue;
                }
            };

            match ws_msg.msg_type.as_str() {
                START_SESSION_MSG => {
                    if let Ok(payload) =
                        serde_json::from_value::<StartSessionPayload>(ws_msg.payload)
                    {
                        info!("Starting session: {:?}", payload);

                        // 初始化 VolcanoEchoMage
                        let config = VolcanoConfig {
                            app_id: "7900512007".to_string(),
                            token: "y3uH1UFivyu4q6gKnwKKIA3snC3FXiXb".to_string(),
                            cluster: "volcengine_streaming_common".to_string(),
                            audio_format: "raw".to_string(),
                            codec: "raw".to_string(),
                            workflow: "audio_in,resample,partition,vad,fe,decode".to_string(),
                            sample_rate: payload.sample_rate,
                        };

                        let mut volcano_asr = VolcanoEchoMage::new(config);
                        if let Err(e) = volcano_asr.start().await {
                            error!("Failed to start ASR: {}", e);
                            socket
                                .send(Message::Text(
                                    json!({
                                        "code": -1,
                                        "msg": "Failed to start ASR"
                                    })
                                    .to_string()
                                    .into(),
                                ))
                                .await
                                .unwrap();
                            return;
                        }

                        asr = Some(volcano_asr);
                        session_started = true;

                        let started = json!({
                            "type": "session_started"
                        });

                        if let Err(e) = socket.send(Message::Text(started.to_string().into())).await
                        {
                            error!("Failed to send session_started: {}", e);
                            break;
                        }
                    } else {
                        error!("Failed to parse start_session payload");
                        socket
                            .send(Message::Text(
                                json!({
                                    "code": -1,
                                    "msg": "Failed to parse start_session payload"
                                })
                                .to_string()
                                .into(),
                            ))
                            .await
                            .unwrap();
                        return;
                    }
                }
                AUDIO_INPUT_CHUNK_MSG => {
                    if !session_started {
                        error!("Received audio chunk before session start");
                        continue;
                    }

                    if let Value::String(audio_data) = ws_msg.payload {
                        if let Ok(decoded) = BASE64.decode(audio_data.as_bytes()) {
                            if let Some(asr) = &mut asr {
                                if let Err(e) = asr.send_audio(&decoded).await {
                                    error!("Failed to send audio to ASR: {}", e);
                                    continue;
                                }
                                //audio_buffer.extend_from_slice(&decoded);
                            }
                        }
                    }
                }
                AUDIO_INPUT_FINISH_MSG => {
                    if let Some(asr) = &mut asr {
                        match asr.receive_result().await {
                            Ok(text) => {
                                info!("ASR Result: {}", text);

                                // 创建Chat实例并处理文本
                                let mut chat = Chat::new(
                                    "default_user".to_string(),
                                    "".to_string(),
                                    role_id.unwrap_or("default_role".to_string()),
                                    &mut app_state,
                                );

                                match chat.on_recv_message(text).await {
                                    Ok(receiver) => {
                                        // 创建TTS配置
                                        let tts_config = TTSConfig {
                                            app_id: "7900512007".to_string(),
                                            token: "y3uH1UFivyu4q6gKnwKKIA3snC3FXiXb".to_string(),
                                            cluster: "volcano_icl".to_string(),
                                            voice_type: "S_TfBFm6r41".to_string(),
                                            enc_format: "pcm".to_string(),
                                        };

                                        // 创建TTS实例
                                        let mut tts = VolcWsTTS::new(tts_config);

                                        // 从Chat的receiver中接收消息并进行TTS转换
                                        while let Ok(chat_response) = receiver.recv() {
                                            if let Err(e) =
                                                tts.init(&chat_response.split_text).await
                                            {
                                                error!("Failed to init TTS: {}", e);
                                                continue;
                                            }

                                            match tts.stream_synthesize().await {
                                                Ok(tts_receiver) => {
                                                    // 处理TTS的音频流
                                                    while let Ok(synth_response) =
                                                        tts_receiver.recv()
                                                    {
                                                        if !synth_response.audio.is_empty() {
                                                            // 将音频数据转换为base64
                                                            let base64_audio = BASE64
                                                                .encode(&synth_response.audio);

                                                            // 发送音频数据到websocket客户端
                                                            if let Err(e) = socket.send(Message::Text(json!({
                                                                "type": "audio_output_chunk",
                                                                "payload": base64_audio
                                                            }).to_string().into())).await {
                                                                error!("Failed to send audio chunk: {}", e);
                                                                break;
                                                            }
                                                        }

                                                        // 如果是最后一个音频块，并且是最后一条消息
                                                        if synth_response.is_last
                                                            && chat_response.is_end
                                                        {
                                                            if let Err(e) = socket
                                                                .send(Message::Text(json!({
                                                                    "type": "audio_output_finished"
                                                                }).to_string().into()))
                                                                .await
                                                            {
                                                                error!("Failed to send finish signal: {}", e);
                                                            }
                                                            break;
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Failed to synthesize speech: {}", e);
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to process chat: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to get ASR result: {}", e);
                            }
                        }
                    }

                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    break;
                }
                _ => {
                    error!("Unknown message type: {}", ws_msg.msg_type);
                }
            }
        }
    }

    info!("WebSocket connection closed");
}
