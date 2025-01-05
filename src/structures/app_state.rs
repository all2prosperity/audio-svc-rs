use config::Config;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
    pub config: Config,
}

impl AppState {
    pub fn new(db_pool: Pool<ConnectionManager<PgConnection>>, config: Config) -> Self {
        Self { db_pool, config }
    }
}
