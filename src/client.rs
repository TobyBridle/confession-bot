use poise::serenity_prelude::{Client, GatewayIntents, Settings};
use poise::{Framework, FrameworkOptions};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

use crate::{commands::*, Config};

pub async fn start(config: Config) -> anyhow::Result<()> {
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
    let client = Client::builder(config.bot_token.as_str(), intents)
        .framework(framework)
        .cache_settings(cache_settings)
        .data(Arc::new(Data {
            config: RwLock::new(config),
        }))
        .await;
    match client {
        Ok(mut client) => {
            if let Err(e) = client.start().await {
                warn!("Client error: {:?}", e);
                panic!();
            }
        }
        Err(e) => {
            warn!("Error creating client: {:?}", e);
            panic!();
        }
    }

    Ok(())
}
