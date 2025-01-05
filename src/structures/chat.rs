

pub struct Chat {
    user_id: i32,
    session_id: i32,
}

impl Chat {
    pub fn new(user_id: i32, session_id: i32) -> Self {
        Self { user_id, session_id }
    }

    pub fn on_recv_message(&self, message: String) {
        println!("recv message: {}", message);

        
    }
}
