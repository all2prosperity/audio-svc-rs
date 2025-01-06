use std::time::SystemTime;

use crate::models::schema;
//use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::sections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Section {
    pub section_id: String,
    pub session_id: String,
    pub user_message: String,
    pub assistant_message: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
