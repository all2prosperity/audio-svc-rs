use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use oz_server::handlers::chat;
use oz_server::utils::mqtt;
use oz_server::{config::OZ_SERVER_CONFIG, structures::AppState};

pub async fn test_llm() {
    // build our application with a single route
    let database_url = OZ_SERVER_CONFIG.get::<String>("database_url").unwrap();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // 创建 AppState
    let mut app_state = AppState::new(pool, OZ_SERVER_CONFIG.clone());

    let mut chat = chat::Chat::new(
        "1".to_string(),
        "4322f33b-3cac-49e4-8310-0584e9608220".to_string(),
        "1".to_string(),
        app_state.db_pool.clone(),
    );
    let mut receiver = chat
        .on_recv_message("那你给我讲讲炉石规则吧".to_string())
        .await
        .unwrap();

    while let Some(msg) = receiver.recv().await {
        println!("msg: {:?}", msg);
    }
}

#[tokio::main]
async fn main() {
    mqtt::publish_event(
        "test".to_string(),
        "4322f33b-3cac-49e4-8310-0584e9608220".to_string(),
    )
    .await
    .unwrap();

    mqtt::publish_message(
        "test".to_string(),
        "test".to_string(),
        "4322f33b-3cac-49e4-8310-0584e9608220".to_string(),
    )
    .await
    .unwrap();
}
