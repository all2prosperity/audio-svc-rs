pub mod mqtt;
use crate::models::establish_connection;
use crate::models::role::Role;
use crate::models::schema;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use log::{error, info};
use std::time::SystemTime;

pub fn insert_default_role() {
    let conn = &mut establish_connection();

    let role = Role {
        id: "1".to_string(),
        is_default: true,
        created_by: "".to_string(),
        name: "default".to_string(),
        prompt: "你是一个炉石传说高手，我会问你炉石传说相关问题".to_string(),
        picture_url: "".to_string(),
        voice_id: "".to_string(),
        audition_url: "".to_string(),
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    };

    match diesel::insert_into(schema::roles::table)
        .values(&role)
        .returning(Role::as_returning())
        .get_result(conn)
    {
        Ok(_) => info!("Inserted default role"),
        Err(e) => error!("Error inserting default role: {}", e),
    }
}

pub fn gen_new_id() -> String {
    xid::new().to_string()
}
