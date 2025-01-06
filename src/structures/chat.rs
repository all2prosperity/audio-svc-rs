use crate::models::{establish_connection, role};
use crate::models::schema::roles::dsl;

use diesel::QueryDsl;
use std::iter::Iterator;
use diesel::SelectableHelper;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;

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

    pub fn on_recv_message(&mut self, message: String) {
        println!("recv message: {}", message);
        if self.session_id == "" {
            self.session_id = uuid::Uuid::new_v4().to_string();
        }
        
        let conn = &mut establish_connection();

        let results = dsl::roles
            .filter(dsl::role_id.eq(self.role_id.clone()))
            .limit(1)
            .select(role::Role::as_select())
            .load(conn)
            .expect("Error loading roles");
    }
}
