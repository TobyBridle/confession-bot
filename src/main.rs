use log::warn;
use std::env;

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
