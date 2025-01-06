use crate::models::{establish_connection, Role};

pub struct Chat {
    xid: String,
    session_id: String,
    role_id: String,
}

impl Chat {
    pub fn new(xid: String, session_id: String, role_id: String) -> Self {
        Self { xid, session_id, role_id }
    }

    pub fn on_recv_message(&mut self, message: String) {
        println!("recv message: {}", message);
        if self.session_id == "" {
            self.session_id = uuid::Uuid::new_v4().to_string();
        }
        
        let conn = establish_connection();

        let results = roles
            .filter(role_id.eq(self.role_id))
            .limit(1)
            .select(Role)
            .load(&conn)
            .expect("Error loading roles");
    }
}
