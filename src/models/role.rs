use std::time::SystemTime;

use crate::models::schema;
//use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub id: String,
    pub is_default: bool,
    pub created_by: String,
    pub name: String,
    pub picture_url: String,
    pub voice_id: String,
    pub audition_url: String,
    pub prompt: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
