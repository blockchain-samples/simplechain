use std::thread;
use r2d2::{Config, Pool};
use r2d2_postgres::{TlsMode, PostgresConnectionManager};

use errors::ServerError;

pub fn get_db_pool() -> Result<Pool<PostgresConnectionManager>, ServerError> {
    let config = Config::default();
    let manager = PostgresConnectionManager::new(
        "postgres://mgul@localhost/blockchain",
        TlsMode::None
    ).unwrap();

    match Pool::new(config, manager) {
        Ok(pool) => Ok(pool),
        Err(e) => Err(ServerError::DatabaseError) // maybe just panic! as we can't establish a connection to database
    }
}
