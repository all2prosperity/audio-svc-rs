use crate::config::OZ_SERVER_CONFIG;
use crate::constant::*;
use crate::json::chat_history_response::{ChatHistoryResponse, History, Payload};
use crate::json::chat_session_history::{
    ChatSessionHistoryRequest, ChatSessionHistoryResponse, History as ChatSessionHistoryHistory,
};
use crate::json::openai_response::OpenAIResponse;
use crate::json::role::AddRoleRequest;
use crate::models::role;
use crate::models::schema;
use crate::models::schema::roles::dsl;
use crate::models::section::Section;
use crate::models::session::Session;
use crate::structures::app_error::AppError;
use crate::structures::app_state::AppState;
use crate::utils;
use crate::utils::mqtt;
use anyhow::Result;
use async_openai::types::ChatCompletionRequestMessage;
use async_openai::{
    config,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use log::debug;
use std::time::SystemTime;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use regex::Regex;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct ChatResponse {
    pub split_text: String,
    pub is_end: bool,
}

use crate::json::chat::ChatHistoryRequest;

pub struct Chat<'a> {
    user_id: String,
    session_id: String,
    role_id: String,
    app_state: &'a mut AppState,
}

impl<'a> Chat<'a> {
    pub fn new(
        user_id: String,
        session_id: String,
        role_id: String,
        app_state: &'a mut AppState,
    ) -> Self {
        Self {
            user_id,
            session_id,
            role_id,
            app_state,
        }
    }

    async fn finish_insert_session(&self) -> Result<String> {
        println!("finish_insert_session {:?}", self.user_id);
        let session = Session {
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            role_id: self.role_id.clone(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        diesel::insert_into(schema::sessions::table)
            .values(&session)
            .execute(&mut self.app_state.db_pool.get()?)?;
        Ok("".to_string())
    }

    async fn finish_insert_message(
        &self,
        message: String,
        assistant_message: String,
    ) -> Result<String> {
        let section = Section {
            section_id: utils::gen_new_id(),
            session_id: self.session_id.clone(),
            user_message: message,
            assistant_message: assistant_message,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        diesel::insert_into(schema::sections::table)
            .values(&section)
            .execute(&mut self.app_state.db_pool.get()?)?;
        Ok("".to_string())
    }

    async fn fill_message_by_session_id(
        &self,
        messages: &mut Vec<ChatCompletionRequestMessage>,
        session_id: String,
    ) -> Result<()> {
        let sections = schema::sections::table
            .filter(schema::sections::session_id.eq(session_id))
            .order(schema::sections::created_at.desc())
            .limit(SECTION_LIMIT)
            .select(Section::as_select())
            .load(&mut self.app_state.db_pool.get()?)?;

        for section in sections.iter().rev() {
            messages.push(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(section.user_message.clone())
                    .build()?
                    .into(),
            );

            messages.push(
                ChatCompletionRequestAssistantMessageArgs::default()
                    .content(section.assistant_message.clone())
                    .build()?
                    .into(),
            );
        }

        Ok(())
    }

    async fn check_need_new_session(&self) -> Result<bool> {
        let session = schema::sessions::table
            .filter(schema::sessions::session_id.eq(self.session_id.clone()))
            .select(Session::as_select())
            .load(&mut self.app_state.db_pool.get()?)?;

        if session.is_empty() {
            return Ok(true);
        }

        let last_session = session.last().ok_or(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Session not found",
        )))?;
        if last_session.role_id != self.role_id {
            return Ok(true);
        }

        Ok(false)
    }

    fn generate_open_client(&self) -> Client<config::OpenAIConfig> {
        let config = config::OpenAIConfig::new()
            .with_api_base(API_BASE_URL)
            .with_api_key(OZ_SERVER_CONFIG.get::<String>(OPEN_API_KEY).unwrap());

        Client::with_config(config)
    }

    async fn generate_session_title(
        &mut self,
        mut messages: Vec<ChatCompletionRequestMessage>,
        device_message: String,
    ) -> Result<String> {
        let client = self.generate_open_client();

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(PROMPT_GENERATE_SESSION_TITLE.to_string())
                .build()?
                .into(),
        );

        messages.push(
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(device_message)
                .build()?
                .into(),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(MAX_TOKENS)
            .model("deepseek-chat")
            .messages(messages)
            .build()?;

        let response = client.chat().create(request).await?;

        let openai_response =
            serde_json::from_str::<OpenAIResponse>(&serde_json::to_string(&response)?)?;

        let title = openai_response.choices[0].message.content.clone();

        Ok(title)
    }

    async fn save_session_title(&self, title: String) -> Result<(), anyhow::Error> {
        diesel::update(
            schema::sessions::table
                .filter(schema::sessions::session_id.eq(self.session_id.clone())),
        )
        .set(schema::sessions::title.eq(title))
        .execute(&mut self.app_state.db_pool.get()?)?;

        Ok(())
    }
}

async fn send_split_message(message: String, sender: Sender<ChatResponse>) {
    let re = Regex::new(r"(,|\\.|，|。|\n\n)").unwrap();
    let split_text = re.split(&message).collect::<Vec<&str>>();
    for text in split_text {
        debug!("send_split_message: {:?}", text);
        let _ = sender
            .send(ChatResponse {
                split_text: text.to_string(),
                is_end: false,
            })
            .await;
    }
    let _ = sender
        .send(ChatResponse {
            split_text: "".to_string(),
            is_end: true,
        })
        .await;
}

impl<'a> Chat<'a> {
    pub async fn on_recv_message(&mut self, message: String) -> Result<Receiver<ChatResponse>> {
        println!("recv message: {}", message);

        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        let mut is_first = false;
        if self.session_id == "" {
            self.session_id = utils::gen_new_id();
            is_first = true;
        } else {
            if self.check_need_new_session().await? {
                self.session_id = utils::gen_new_id();
                is_first = true;
            }
        }

        let results = dsl::roles
            .filter(dsl::id.eq(self.role_id.clone()))
            .limit(1)
            .select(role::Role::as_select())
            .load(&mut self.app_state.db_pool.get()?)?;

        let role = results.first().ok_or(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Role not found",
        )))?;

        let client = self.generate_open_client();

        let mut messages = Vec::new();
        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(role.prompt.clone())
                .build()?
                .into(),
        );

        if !is_first {
            let _ = self
                .fill_message_by_session_id(&mut messages, self.session_id.clone())
                .await;
        }

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(message.clone())
                .build()?
                .into(),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(MAX_TOKENS)
            .model("deepseek-chat")
            .messages(messages.clone())
            .build()?;

        let response = client.chat().create(request).await?;

        let openai_response =
            serde_json::from_str::<OpenAIResponse>(&serde_json::to_string(&response)?)?;

        if is_first {
            let _ = self.finish_insert_session().await;
        }

        let device_message = openai_response.choices[0].message.content.clone();

        let _ = self
            .finish_insert_message(message.clone(), device_message.clone())
            .await;

        let device_message_clone_for_split = device_message.clone();
        tokio::spawn(async move {
            send_split_message(device_message_clone_for_split, sender).await;
        });

        let self_message = message.clone();
        let device_id = self.user_id.clone();
        let device_message_clone_for_mqtt = device_message.clone();

        tokio::spawn(async move {
            let _ =
                mqtt::publish_message(self_message, device_message_clone_for_mqtt, device_id).await;
        });

        if is_first {
            let title = self
                .generate_session_title(messages, device_message)
                .await?;
            self.save_session_title(title).await?;
        }

        Ok(receiver)
    }

    pub async fn get_chat_history(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<ChatHistoryResponse, Box<dyn std::error::Error>> {
        let sessions = schema::sessions::table
            .filter(schema::sessions::user_id.eq(self.user_id.clone()))
            .order(schema::sessions::updated_at.desc())
            .limit(page_size)
            .offset(page * page_size)
            .select(Session::as_select())
            .load(&mut self.app_state.db_pool.get()?)?;

        let mut history = Vec::new();
        let total = sessions.len() as i64;
        for session in sessions {
            history.push(History {
                chat_id: session.session_id.clone(),
                title: session.role_id.clone(),
                role_name: session.role_id.clone(),
            });
        }

        Ok(ChatHistoryResponse {
            code: 0,
            msg: "".to_string(),
            payload: Payload {
                history,
                page,
                total,
            },
        })
    }

    pub async fn get_chat_session_history(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<ChatSessionHistoryResponse, Box<dyn std::error::Error>> {
        let sections = schema::sections::table
            .filter(schema::sections::session_id.eq(self.session_id.clone()))
            .order(schema::sections::updated_at.desc())
            .limit(page_size)
            .offset(page * page_size)
            .select(Section::as_select())
            .load(&mut self.app_state.db_pool.get()?)?;

        let mut history = Vec::new();
        let total = sections.len() as i64;
        for section in sections {
            history.push(ChatSessionHistoryHistory {
                id: section.section_id.clone(),
                user: section.user_message.clone(),
                assistant: section.assistant_message.clone(),
            });
        }

        Ok(ChatSessionHistoryResponse {
            code: 0,
            msg: "".to_string(),
            history,
            page,
            limit: page_size,
            total,
        })
    }

    pub async fn add_role(
        &self,
        name: String,
        prompt: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let role = role::Role {
            id: utils::gen_new_id(),
            is_default: false,
            created_by: self.user_id.clone(),
            name,
            picture_url: "".to_string(),
            voice_id: "".to_string(),
            audition_url: "".to_string(),
            prompt,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        diesel::insert_into(schema::roles::table)
            .values(&role)
            .execute(&mut self.app_state.db_pool.get()?)?;

        Ok(())
    }
}

//目前不需要这个handler
// pub async fn chat(
//     State(mut app_state): State<AppState>,
//     headers: HeaderMap,
//     Json(request): Json<ChatRequest>,
// ) -> impl IntoResponse {
//     println!(
//         "{}, {:?}",
//         serde_json::to_string(&request).unwrap(),
//         headers
//     );

//     // let mut chat = Chat::new(
//     //     request.user_id,
//     //     request.session_id,
//     //     request.role_id,
//     //     &mut app_state,
//     // );
//     // if let Ok(response) = chat.on_recv_message(request.message).await {
//     //     return Json(response).into_response();
//     // } else {
//     //     return StatusCode::INTERNAL_SERVER_ERROR.into_response();
//     // }
// }

pub async fn chat_history(
    mut app_state: State<AppState>,
    Query(query): Query<ChatHistoryRequest>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    println!("hello {:?}, {:?}", query, headers);
    let user_id = headers
        .get("x-oz-user-id")
        .ok_or(AppError(anyhow::anyhow!("User id not found")))?
        .to_str()
        .unwrap_or("");
    let chat = Chat::new(
        user_id.to_string(),
        "".to_string(),
        "".to_string(),
        &mut app_state,
    );
    if let Ok(response) = chat.get_chat_history(query.offset, query.limit).await {
        return Ok(Json(response).into_response());
    } else {
        return Err(AppError(anyhow::anyhow!("Failed to get chat history")));
    }
}

pub async fn chat_session_history(
    mut app_state: State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ChatSessionHistoryRequest>,
) -> Result<impl IntoResponse, AppError> {
    println!("hello {:?}, {:?}", request, headers);
    let user_id = headers
        .get("x-oz-user-id")
        .ok_or(AppError(anyhow::anyhow!("User id not found")))?
        .to_str()
        .unwrap_or("");

    let chat = Chat::new(
        user_id.to_string(),
        request.chat_id,
        "".to_string(),
        &mut app_state,
    );
    if let Ok(response) = chat
        .get_chat_session_history(request.offset, request.limit)
        .await
    {
        return Ok(Json(response).into_response());
    } else {
        return Err(AppError(anyhow::anyhow!(
            "Failed to get chat session history"
        )));
    }
}

pub async fn add_role(
    State(mut app_state): State<AppState>,
    header: HeaderMap,
    Json(request): Json<AddRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = header
        .get("x-oz-user-id")
        .ok_or(AppError(anyhow::anyhow!("User id not found")))?
        .to_str()
        .unwrap_or("");

    let chat = Chat::new(
        user_id.to_string(),
        "".to_string(),
        "".to_string(),
        &mut app_state,
    );
    let _ = chat.add_role(request.name, request.prompt).await;

    Ok(StatusCode::OK.into_response())
}

#[cfg(test)]
mod tests {
    use diesel::{
        r2d2::{ConnectionManager, Pool},
        PgConnection,
    };

    use super::*;
    // use log::{Builder, LevelFilter, Record};
    // use std::io::Write;
    // use std::time::Local;

    #[tokio::test]
    async fn test_chat() {
        // Builder::from_default_env()
        //     .format(|buf, record| {
        //     writeln!(
        //         buf,
        //         "{} [{}] - {}",
        //         Local::now().format("%Y-%m-%d %H:%M:%S"),
        //         record.level(),
        //         record.args()
        //     )
        // })
        // .write_style(WriteStyle::Always)
        // .filter_level(log::LevelFilter::Debug)
        // .filter_module("oz_server", log::LevelFilter::Debug)
        // .init();

        // 设置数据库连接池
        //let database_url = OZ_SERVER_CONFIG.get::<String>("database_url").unwrap();
        let manager =
            ConnectionManager::<PgConnection>::new("postgres://oz_liutong:1157039@localhost/oz_db");
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        let mut app_state = AppState::new(pool, OZ_SERVER_CONFIG.clone());

        let mut chat = Chat::new(
            "default_user".to_string(),
            "".to_string(),
            "default_role".to_string(),
            &mut app_state,
        );
        let mut receiver = chat.on_recv_message("hello".to_string()).await.unwrap();
        loop {
            println!("receiver ready to recv");
            let chat_response = receiver.recv().await;
            if chat_response.is_none() {
                break;
            }
            let chat_response = chat_response.unwrap();
            println!("chat_response: {:?}", chat_response.split_text);
        }
    }
}
