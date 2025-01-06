use std::time::SystemTime;

use crate::models::schema;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::user_role)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UseRole {
    pub id: String,
    pub role_id: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
