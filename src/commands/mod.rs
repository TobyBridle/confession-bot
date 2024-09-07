use log::{error, info};
use poise::{serenity_prelude as serenity, FrameworkError};
use serenity::FullEvent;

pub mod confess;

pub type Data = ();
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn check(ctx: Context<'_>) -> Result<bool, Error> {
    Ok(!ctx.author().bot)
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match &event {
        FullEvent::Ready { data_about_bot, .. } => {
            info!(
                "Logged in as: {}. Currently observing {} guild(s)",
                data_about_bot.user.name,
                data_about_bot.guilds.len()
            );
        }
        FullEvent::Message { new_message } => {
            if new_message.mentions_user_id(&framework.bot_id) {
                println!("WE GOT MENTIONEEEEEDDDDD")
            }
        }
        _ => {}
    }
    Ok(())
}

pub async fn on_error(err: FrameworkError<'_, Data, Error>) {
    match err {
        FrameworkError::UnknownCommand {
            ctx,
            msg,
            prefix,
            msg_content,
            framework,
            invocation_data,
            trigger,
            ..
        } if msg.mentions_user_id(framework.bot_id) => {
            return;
        }
        _ => {
            error!(
                "Error occured ({}): {}",
                chrono::Local::now(),
                err.to_string()
            )
        }
    }
}
