use crate::models::role::Role;
use crate::models::establish_connection;
use std::time::SystemTime;
use crate::models::schema;
use diesel::SelectableHelper;
use diesel::RunQueryDsl;


pub fn insert_default_role() {
    let conn = &mut establish_connection();

    let role = Role {
        id: "1".to_string(),
        name: "default".to_string(),
        prompt: "你是一个炉石传说高手，我会问你炉石传说相关问题".to_string(),
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    };

    diesel::insert_into(schema::roles::table)
        .values(&role)
        .returning(Role::as_returning())
        .get_result(conn);
}


pub fn genNewId() -> String {
    uuid::Uuid::new_v4().to_string()
}