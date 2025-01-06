use std::time::SystemTime;

use crate::models::schema;
//use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub role_id: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
