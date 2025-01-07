use oz_server::utils::insert_default_role;
use oz_server::structures::chat;

#[tokio::test]
async fn test_model() {
    insert_default_role();
}


#[tokio::test]
async fn test_chat() {
    chat::Chat::new("1".to_string(), "".to_string(), "1".to_string())
        .on_recv_message("你好".to_string()).await.unwrap();
}