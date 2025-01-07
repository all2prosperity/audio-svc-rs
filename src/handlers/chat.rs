use crate::models::schema::roles::dsl;
use crate::models::{establish_connection, role};
use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client, config,
};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use crate::config::OZ_SERVER_CONFIG;
use crate::json::openai_response::OpenAIResponse;
use crate::models::schema;
use crate::models::section::Section;
use std::time::SystemTime;
use async_openai::types::ChatCompletionRequestMessage;
use crate::models::session::Session;
use crate::constant::*;

use axum::{
    extract::{Extension, Request},
    http,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

pub struct Chat {
    user_id: String,
    session_id: String,
    role_id: String,
}

impl Chat {
    pub fn new(user_id: String, session_id: String, role_id: String) -> Self {
        Self {
            user_id,
            session_id,
            role_id,
        }
    }

    async fn finish_insert_session(&self, conn: &mut diesel::PgConnection) -> Result<String, Box<dyn std::error::Error>> {
        let session = Session {
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            role_id: self.role_id.clone(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        diesel::insert_into(schema::sessions::table)
            .values(&session)
            .execute(conn)?;
        Ok("".to_string())
    }

    async fn finish_insert_message(&self, conn: &mut diesel::PgConnection, message: String, assistant_message: String) -> Result<String, Box<dyn std::error::Error>> {
        let section = Section {
            section_id: uuid::Uuid::new_v4().to_string(),
            session_id: self.session_id.clone(),
            user_message: message,
            assistant_message: assistant_message,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        diesel::insert_into(schema::sections::table)
            .values(&section)
            .execute(conn)?;
        Ok("".to_string())
    }

    async fn fill_message_by_session_id(&self, conn: &mut diesel::PgConnection, messages: &mut Vec<ChatCompletionRequestMessage>, session_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let sections = schema::sections::table
            .filter(schema::sections::session_id.eq(session_id))
            .order(schema::sections::created_at.desc())
            .limit(SECTION_LIMIT)
            .select(Section::as_select())
            .load(conn)?;

        for section in sections {
            messages.push(ChatCompletionRequestUserMessageArgs::default()
                .content(section.user_message.clone())
                .build()?
                .into());

            messages.push(ChatCompletionRequestAssistantMessageArgs::default()
                .content(section.assistant_message.clone())
                .build()?
                .into());
        }

        Ok(())
    }

    async fn check_need_new_session(&self, conn: &mut diesel::PgConnection) -> Result<bool, Box<dyn std::error::Error>> {
        let session = schema::sessions::table
            .filter(schema::sessions::session_id.eq(self.session_id.clone()))
            .select(Session::as_select())
            .load(conn)?;

        if session.is_empty() {
            return Ok(true);
        }

        let last_session = session.last().ok_or(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Session not found")))?;
        if last_session.role_id != self.role_id {
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn on_recv_message(&mut self, message: String) -> Result<String, Box<dyn std::error::Error>> {
        println!("recv message: {}", message);
        let conn = &mut establish_connection();

        let mut is_first = false;
        if self.session_id == "" {
            self.session_id = uuid::Uuid::new_v4().to_string();
            is_first = true;
        }
        else {
            if self.check_need_new_session(conn).await? {
                self.session_id = uuid::Uuid::new_v4().to_string();
                is_first = true;
            }
        }

        let results = dsl::roles
            .filter(dsl::id.eq(self.role_id.clone()))
            .limit(1)
            .select(role::Role::as_select())
            .load(conn)?;

        let role = results.first().ok_or(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Role not found")))?;

        let config = config::OpenAIConfig::new()
        .with_api_base(API_BASE_URL)
        .with_api_key(OZ_SERVER_CONFIG.get::<String>(OPEN_API_KEY).unwrap());

        let client = Client::with_config(config);

        let mut messages = Vec::new();
        messages.push(ChatCompletionRequestSystemMessageArgs::default()
            .content(role.prompt.clone())
            .build()?
            .into());

        if !is_first {
            self.fill_message_by_session_id(conn, &mut messages, self.session_id.clone()).await;
        }

        messages.push(ChatCompletionRequestUserMessageArgs::default()
            .content(message.clone())
            .build()?
            .into());

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u32)
            .model("deepseek-chat")
            .messages(messages)
            .build()?;

        println!("{}", serde_json::to_string(&request).unwrap());

        let response = client.chat().create(request).await?;

        println!("{}", serde_json::to_string(&response).unwrap());

        let openai_response = serde_json::from_str::<OpenAIResponse>(&serde_json::to_string(&response).unwrap()).unwrap();

        if is_first {
            self.finish_insert_session(conn).await;
        }

        self.finish_insert_message(conn, message, openai_response.choices[0].message.content.clone()).await;

        Ok("".to_string())
    }
}

pub async fn chat(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}
