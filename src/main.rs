use anyhow::Context;
use std::env;
use tokio::fs;
use tracing::{error, subscriber};
use tracing_subscriber::FmtSubscriber;

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
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::new();
    subscriber::set_global_default(subscriber)?;
    let _ = dotenvy::dotenv();
    let config = Config {
        bot_token: env::var("BOT_TOKEN").context("BOT_TOKEN not set")?,
        db_url: env::var("DATABASE_URL").context("DATABASE_URL not set")?,
    };
    if let Ok(meta) = fs::metadata(&config.db_url).await {
        if !meta.is_file() && !meta.is_symlink() {
            error!(
                "Expected file at {}. Found {:?}",
                &config.db_url,
                meta.file_type()
            )
        }
    } else {
        error!("Cannot find file at {}", &config.db_url)
    }
    client::start(config).await?;
    Ok(())
}
