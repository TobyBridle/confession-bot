use poise::{serenity_prelude::CreateEmbed, CreateReply};

use crate::{
    commands::{Context, Error},
    db_impl::guilds,
};

/// Post a confession into the confession channel
#[poise::command(slash_command, ephemeral, rename = "confess")]
pub async fn confession(
    ctx: Context<'_>,
    #[description = "The confession text content"] content: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let config = data.config.read().await;
    let guild =
        guilds::get_guild(config.db_url.clone(), ctx.guild_id().unwrap().to_string()).await?;
    if guild.is_none() {
        ctx.send(CreateReply::new().embed(
            CreateEmbed::new()
                .color(0xFF0000)
                .title("Database Error")
                .description("The current Guild does not exist within the Bot's Database. Please attempt to run the command again.")
        ))
            .await?;
        guilds::insert_guild(config.db_url.clone(), ctx.guild_id().unwrap().to_string()).await?;
        return Ok(());
    }
    Ok(())
}
