use diesel::prelude::*;
use crate::models::schema;
use chrono::{DateTime, Utc};

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub role_id: String,
    pub prompt: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
