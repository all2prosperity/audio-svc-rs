use std::time::SystemTime;

use crate::models::schema;
//use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub role_id: String,
    pub prompt: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
