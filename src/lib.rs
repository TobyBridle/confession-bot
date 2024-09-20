pub mod models;
pub mod schema;

use diesel::{sqlite::SqliteConnection, Connection};

pub fn establish_connection(db_url: String) -> SqliteConnection {
    SqliteConnection::establish(&db_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
}
