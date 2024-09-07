use log::info;
use poise::serenity_prelude::{self as serenity, Client, FullEvent, GatewayIntents, Settings};
use poise::{builtins, Framework, FrameworkOptions};

use crate::commands::*;

pub async fn start(bot_token: String) {
    let framework = Framework::builder()
        .setup(|ctx, _, framework| {
            Box::pin(async move {
                builtins::register_globally(&ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .options(FrameworkOptions {
            commands: vec![confess::confession()],
            event_handler: |ctx, event, _ctx, _a| Box::pin(event_handler(ctx, event, _ctx, _a)),
            on_error: |err| Box::pin(on_error(err)),
            ..Default::default()
        })
        .build();
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let cache_settings = Settings::default();
    let client = Client::builder(bot_token, intents)
        .framework(framework)
        .cache_settings(cache_settings)
        .await;
    // We WANT to crash if something goes wrong.
    client.unwrap().start().await.unwrap();
}
