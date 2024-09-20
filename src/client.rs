use std::sync::Arc;

use poise::serenity_prelude::{Client, GatewayIntents, Settings};
use poise::{Framework, FrameworkOptions};
use tokio::sync::RwLock;

use crate::{commands::*, Config};

pub async fn start(config: &Config) {
    let framework = Framework::builder()
        // .setup(|ctx, _, framework| {
        //     Box::pin(async move {
        //         Ok(())
        //     })
        // })
        .options(FrameworkOptions {
            commands: vec![confess::confession(), config::config_guild()],
            event_handler: |ctx, event| Box::pin(event_handler(ctx, event)),
            on_error: |err| Box::pin(on_error(err)),
            ..Default::default()
        })
        .build();
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let cache_settings = Settings::default();
    let config = config.clone();
    let client = Client::builder(config.bot_token.clone().as_str(), intents)
        .framework(framework)
        .cache_settings(cache_settings)
        .data(Arc::new(Data {
            config: RwLock::new(config),
        }))
        .await;
    // We WANT to crash if something goes wrong.
    client.unwrap().start().await.unwrap();
}
