use confession_bot_rs::{
    establish_connection,
    models::{Author, Guild, NewAuthor, NewConfession, NewGuild},
    schema::{authors, confession, guild},
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use log::warn;
use std::{
    env,
    sync::{Arc, Mutex},
};

mod client;
mod commands;
mod db_impl;
mod models;
mod schema;

#[derive(Clone)]
struct Config {
    db_url: String,
    bot_token: String,
}

#[tokio::main]
async fn main() -> Result<(), diesel::result::Error> {
    pretty_env_logger::init();
    if dotenvy::dotenv().is_err() {
        warn!("Warning: .env file not found.")
    }
    let ref config = Config {
        bot_token: env::var("BOT_TOKEN").expect("BOT_TOKEN set"),
        db_url: env::var("DATABASE_URL").expect("DATABASE_URL set"),
    };
    client::start(config).await;
    Ok(())
}

// let author = diesel::insert_into(authors::table)
//     .values(&NewAuthor {
//         hash: "test".to_string(),
//     })
//     .returning(authors::id)
//     .get_result(&mut conn)?;
// let guild_id: &String = &diesel::insert_into(guild::table)
//     .values(&NewGuild {
//         guild_id: "work".to_string(),
//         config: "{}".to_string(),
//     })
//     .returning(guild::guild_id)
//     .get_result(&mut conn)?;
// diesel::insert_into(confession::table)
//     .values(&NewConfession {
//         author,
//         guild_id,
//         content: &"This is a test".to_string(),
//         message_id: &"2131312".to_string(),
//     })
//     .execute(&mut conn)?;
// let res: Vec<Guild> = guild::table.select(Guild::as_select()).load(&mut conn)?;
