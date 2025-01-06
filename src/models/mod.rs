pub mod role;
pub mod schema;
pub mod user_role;
use diesel::prelude::*;

use crate::config::OZ_SERVER_CONFIG;

pub fn test_model() {
    println!("test_model");
}

pub fn establish_connection() -> PgConnection {
    let database_url = OZ_SERVER_CONFIG.get::<String>("database_url").unwrap();
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
