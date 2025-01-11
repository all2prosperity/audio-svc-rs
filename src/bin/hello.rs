use axum::{
    routing::get,
    Router,
};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use oz_server::{config::OZ_SERVER_CONFIG, structures::AppState};
use oz_server::handlers::chat;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let database_url = OZ_SERVER_CONFIG.get::<String>("database_url").unwrap();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // 创建 AppState
    let mut app_state = AppState::new(pool, OZ_SERVER_CONFIG.clone());

    let mut chat = chat::Chat::new("1".to_string(), "4322f33b-3cac-49e4-8310-0584e9608220".to_string(), "1".to_string(), &mut app_state);
    let receiver = chat.on_recv_message("那你给我讲讲炉石规则吧".to_string()).await.unwrap();

    for msg in receiver {
        println!("msg: {:?}", msg);
    }
}
