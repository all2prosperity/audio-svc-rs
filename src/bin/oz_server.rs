use axum::{extract::State, routing::get, Router};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use oz_server::{config::OZ_SERVER_CONFIG, structures::AppState};

async fn health_check(State(_state): State<AppState>) -> &'static str {
    "OK"
}

async fn setup_router(app_state: AppState) -> Router {
    Router::new()
        .route("/api/roles", get(get_roles))
        .route("/api/role/switch", post(switch_role))
        .route("/health", get(health_check))
        .with_state(app_state)
}

async fn _main() {
    // 设置数据库连接池
    let database_url = OZ_SERVER_CONFIG.get::<String>("database_url").unwrap();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // 创建 AppState
    let app_state = AppState::new(pool, OZ_SERVER_CONFIG.clone());

    // 设置路由
    let app = setup_router(app_state).await;

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

#[tokio::main]
async fn main() {
    _main().await;
}
