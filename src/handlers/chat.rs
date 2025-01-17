use crate::config::OZ_SERVER_CONFIG;
use crate::constant::*;
use crate::constant::*;
use crate::json::chat_history_response::{ChatHistoryResponse, History, Payload};
use crate::json::chat_session_history::{
    ChatSessionHistoryRequest, ChatSessionHistoryResponse, History as ChatSessionHistoryHistory,
};
use crate::json::openai_response::OpenAIResponse;
use crate::json::role::AddRoleRequest;
use crate::models::schema;
use crate::models::schema::roles::dsl;
use crate::models::section::Section;
use crate::models::session::Session;
use crate::models::{establish_connection, role};
use crate::structures::app_error::AppError;
use crate::structures::app_state::AppState;
use crate::utils;
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
use std::time::SystemTime;

use axum::{
    extract::{Extension, Query, Request, State},
    http,
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

use crossbeam::channel::{Receiver, Sender};
use regex::Regex;

#[derive(Debug)]
pub struct ChatResponse {
    pub split_text: String,
    pub is_end: bool,
}

use crate::json::chat::{ChatHistoryRequest, ChatRequest};

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

}

async fn send_split_message(message: String, sender: Sender<ChatResponse>) {
    let re = Regex::new(r"(,|\\.|，|。|\n\n)").unwrap();
    let split_text = re.split(&message).collect::<Vec<&str>>();
    for text in split_text {
        sender.send(ChatResponse {
            split_text: text.to_string(),
            is_end: false,
        });
    }
    sender.send(ChatResponse {
        split_text: "".to_string(),
        is_end: true,
    });
}

impl<'a> Chat<'a> {
    pub async fn on_recv_message(&mut self, message: String) -> Result<Receiver<ChatResponse>> {
        println!("recv message: {}", message);

        let (sender, receiver) = crossbeam::channel::unbounded();

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

        let config = config::OpenAIConfig::new()
            .with_api_base(API_BASE_URL)
            .with_api_key(OZ_SERVER_CONFIG.get::<String>(OPEN_API_KEY).unwrap());

        let client = Client::with_config(config);

        let mut messages = Vec::new();
        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(role.prompt.clone())
                .build()?
                .into(),
        );

        if !is_first {
            self.fill_message_by_session_id(&mut messages, self.session_id.clone())
                .await;
        }

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(message.clone())
                .build()?
                .into(),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u32)
            .model("deepseek-chat")
            .messages(messages)
            .build()?;

        println!("{}\n\n", serde_json::to_string(&request).unwrap());

        let response = client.chat().create(request).await?;

        println!("{}", serde_json::to_string(&response).unwrap());

        let openai_response =
            serde_json::from_str::<OpenAIResponse>(&serde_json::to_string(&response).unwrap())
                .unwrap();

        if is_first {
            self.finish_insert_session().await;
        }

        self.finish_insert_message(message, openai_response.choices[0].message.content.clone())
            .await;

        let content = openai_response.choices[0].message.content.clone();
        tokio::spawn(async move {
            send_split_message(content, sender).await;
        });
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
    chat.add_role(request.name, request.prompt).await;

    Ok(StatusCode::OK.into_response())
}
