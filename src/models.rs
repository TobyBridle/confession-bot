use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Identifiable, Selectable, PartialEq)]
#[diesel(table_name = crate::schema::authors)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Author {
    pub id: i32,
    pub hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::authors)]
pub struct NewAuthor {
    pub hash: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct GuildConfig {
    pub delete_vote_min: i32,
    pub expose_vote_min: i32,
    pub expose_vote_role: Option<String>,
    pub role_ping: Option<String>,
}

#[derive(Queryable, Selectable, PartialEq, Clone)]
#[diesel(table_name = crate::schema::guild)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Guild {
    pub guild_id: String,
    pub confession_channel_id: Option<String>,
    pub config: String,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::guild)]
pub struct NewGuild {
    pub guild_id: String,
    pub config: String,
}

#[derive(Queryable, Associations, Selectable, PartialEq, Clone)]
#[diesel(belongs_to(Author, foreign_key = author))]
#[diesel(table_name = crate::schema::confession)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Confession {
    pub id: i32,
    pub guild_id: String,
    pub message_id: String,
    pub content: String,
    pub author: i32,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::confession)]
pub struct NewConfession<'a> {
    pub guild_id: &'a String,
    pub message_id: &'a String,
    pub content: &'a String,
    pub author: i32,
}

#[derive(Queryable, Selectable, Associations, PartialEq)]
#[diesel(belongs_to(Author, foreign_key=author_id))]
#[diesel(belongs_to(Confession, foreign_key=confession_id))]
#[diesel(table_name = crate::schema::delete_votes)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Vote {
    pub id: i32,
    pub confession_id: i32,
    pub author_id: i32,
    pub vote_type: String,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Associations, PartialEq)]
#[diesel(belongs_to(Author, foreign_key=author))]
#[diesel(belongs_to(Guild, foreign_key=guild_id))]
#[diesel(belongs_to(Confession, foreign_key=original_confession_id))]
#[diesel(table_name = crate::schema::replies)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Reply {
    pub id: i32,
    pub guild_id: String,
    pub original_confession_id: i32,
    pub author: i32,
    pub message_id: String,
    pub content: String,
    pub timestamp: chrono::NaiveDateTime,
}
