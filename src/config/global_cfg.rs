use config::{Config, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AsrConfig {
    pub app_id: String,
    pub token: String,
    pub cluster: String,
    pub audio_format: String,
    pub codec: String,
    pub workflow: String,
    pub sample_rate: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TtsConfig {
    pub app_id: String,
    pub token: String,
    pub cluster: String,
    pub voice_type: String,
    pub enc_format: String,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            app_id: "9335493412".to_string(),
            token: "m6oeKE_9QbYQg4EiWunryPUErK81rMRF".to_string(),
            cluster: "volcengine_streaming_common".to_string(),
            audio_format: "mp3".to_string(),
            codec: "raw".to_string(),
            workflow: "audio_in,resample,partition,vad,fe,decode".to_string(),
            sample_rate: 16000,
        }
    }
}

impl AsrConfig {
    pub fn to_toolkit_config(&self) -> llm_audio_toolkit::asr::volc::VolcanoConfig {
        llm_audio_toolkit::asr::volc::VolcanoConfig {
            app_id: self.app_id.clone(),
            token: self.token.clone(),
            cluster: self.cluster.clone(),
            audio_format: self.audio_format.clone(),
            codec: self.codec.clone(),
            workflow: self.workflow.clone(),
            sample_rate: self.sample_rate,
        }
    }
}


impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            app_id: "9335493412".to_string(),
            token: "m6oeKE_9QbYQg4EiWunryPUErK81rMRF".to_string(),
            cluster: "volcano_icl".to_string(),
            voice_type: "S_TfBFm6r41".to_string(),
            enc_format: "mp3".to_string(),
        }
    }
}

impl TtsConfig {
    pub fn to_toolkit_config(&self) -> llm_audio_toolkit::tts::volc::VolcConfig {
        llm_audio_toolkit::tts::volc::VolcConfig {
            app_id: self.app_id.clone(),
            token: self.token.clone(),
            cluster: self.cluster.clone(),
            voice_type: self.voice_type.clone(),
            enc_format: self.enc_format.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub openai_api_key: String,
    pub database_url: String,

    pub asr_config: AsrConfig,
    pub tts_config: TtsConfig,
}

impl GlobalConfig {
    pub fn load() -> Self {
        let config = Config::builder()
            .add_source(File::with_name(".config.json"))
            .build()
            .unwrap();
        Self {
            openai_api_key: config.get::<String>("openai_api_key").unwrap(),
            database_url: config.get::<String>("database_url").unwrap(),
            asr_config: AsrConfig::default(),
            tts_config: TtsConfig::default(),
        }
    }
}
