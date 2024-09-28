pub mod models;
pub mod schema;

use diesel::{sqlite::SqliteConnection, Connection};

pub fn establish_connection(db_url: String) -> SqliteConnection {
    SqliteConnection::establish(&db_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
}

#[derive(PartialEq, Eq)]
pub enum VoteType {
    DELETE,
    EXPOSE,
}

pub const DELETE_VOTE_STR: &str = "delete_vote";
pub const EXPOSE_VOTE_STR: &str = "expose_vote";
impl Into<String> for VoteType {
    fn into(self) -> String {
        match self {
            VoteType::DELETE => DELETE_VOTE_STR.to_string(),
            VoteType::EXPOSE => EXPOSE_VOTE_STR.to_string(),
        }
    }
}

impl Into<VoteType> for String {
    fn into(self) -> VoteType {
        return match self.to_lowercase().as_str() {
            DELETE_VOTE_STR => VoteType::DELETE,
            EXPOSE_VOTE_STR => VoteType::EXPOSE,
            _ => panic!("Could not convert {} into a VoteType.", self),
        };
    }
}
