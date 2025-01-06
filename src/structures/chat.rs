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

pub struct Chat {
    xid: String,
    session_id: String,
    role_id: String,
}

impl Chat {
    pub fn new(xid: String, session_id: String, role_id: String) -> Self {
        Self {
            xid,
            session_id,
            role_id,
        }
    }

    async fn finish_insert_message(&mut self, conn: &mut diesel::PgConnection, message: String, assistant_message: String) -> Result<String, Box<dyn std::error::Error>> {
        Ok("".to_string())
    }

    pub async fn on_recv_message(&mut self, message: String) -> Result<String, Box<dyn std::error::Error>> {
        println!("recv message: {}", message);
        let mut is_first = false;
        if self.session_id == "" {
            self.session_id = uuid::Uuid::new_v4().to_string();
            is_first = true;
        }

        let conn = &mut establish_connection();

        let results = dsl::roles
            .filter(dsl::id.eq(self.role_id.clone()))
            .limit(1)
            .select(role::Role::as_select())
            .load(conn)
            .expect("Error loading roles");

        let role = results.first().unwrap();

        let config = config::OpenAIConfig::new()
        .with_api_base("https://api.deepseek.com")
        .with_api_key(OZ_SERVER_CONFIG.get::<String>("OPEN_API_KEY").unwrap());

        let client = Client::with_config(config);

        let mut messages = Vec::new();
        messages.push(ChatCompletionRequestSystemMessageArgs::default()
            .content(role.prompt.clone())
            .build()?
            .into());

        if !is_first {
        }

        messages.push(ChatCompletionRequestUserMessageArgs::default()
            .content(message)
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

        Ok("".to_string())
    }
}
