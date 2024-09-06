use diesel::prelude::*;

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
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

#[derive(Queryable, Selectable, Debug, PartialEq)]
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

#[derive(Queryable, Associations, Selectable, Debug, PartialEq, Clone)]
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

#[derive(Queryable, Selectable, Associations, Debug, PartialEq)]
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
