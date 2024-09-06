pub mod models;
pub mod schema;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use diesel::{sqlite::SqliteConnection, Connection};

pub fn establish_connection(db_url: String) -> SqliteConnection {
    SqliteConnection::establish(&db_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
}
