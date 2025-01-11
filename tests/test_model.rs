use oz_server::utils::insert_default_role;
use oz_server::handlers::chat;

#[tokio::test]
async fn test_model() {
    insert_default_role();
}


#[tokio::test]
async fn test_chat() {

    // let mut chat = chat::Chat::new("1".to_string(), "4322f33b-3cac-49e4-8310-0584e9608220".to_string(), "1".to_string(), "1".to_string());
    // chat.on_recv_message("那你给我讲讲炉石规则吧".to_string()).await.unwrap();
}
