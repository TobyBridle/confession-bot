use log::{error, info};
use poise::serenity_prelude::{self as serenity, Client, GatewayIntents, Settings};
use poise::{builtins, Framework, FrameworkError, FrameworkOptions};

type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn on_error(err: FrameworkError<'_, Data, Error>) {
    error!(
        "Error occured ({}): {}",
        chrono::Local::now(),
        err.to_string()
    )
}

#[poise::command(slash_command, ephemeral, check = "check", rename = "confess")]
async fn confession(
    ctx: Context<'_>,
    #[description = "The confession text content"] content: String,
) -> Result<(), Error> {
    ctx.reply("some shi").await?;
    Ok(())
}

async fn check(ctx: Context<'_>) -> Result<bool, Error> {
    Ok(!ctx.author().bot)
}

pub async fn start(bot_token: String) {
    let framework = Framework::builder()
        .setup(|ctx, ready, framework| {
            info!(
                "Logged in as: {}. Currently observing {} guild(s)",
                ready.user.name,
                ready.guilds.len()
            );
            Box::pin(async move {
                builtins::register_globally(&ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .options(FrameworkOptions {
            commands: vec![confession()],
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
