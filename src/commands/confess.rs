use log::error;
use poise::{
    serenity_prelude::{CreateEmbed, CreateMessage},
    CreateReply,
};

use crate::db_impl::guilds;
use crate::{
    commands::{Context, Error},
    db_impl::confessions::{self, insert_confession},
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
    let guild = guild.unwrap();
    if guild.confession_channel_id.is_none() {
        ctx.send(
            CreateReply::new().embed(
                CreateEmbed::new()
                    .title("Guild Error")
                    .description("A confession channel must be set before using `/confess`!\nPlease request a moderator to set one using the `/config` command.")
                    .color(0xFFAA00)
            ),
        )
        .await?;
        return Ok(());
    }
    let channel = guild.confession_channel_id.unwrap();
    let ctx_guild = ctx.guild().unwrap().to_owned();
    let guild_channel = ctx_guild
        .channels
        .iter()
        .find(|c| c.id.to_string() == channel)
        .cloned();
    if guild_channel.is_none() {
        ctx.send(
            CreateReply::new().embed(
                CreateEmbed::new()
                    .color(0xFF0000)
                    .title("Guild Error")
                    .description(format!(
                        "Channel <#{}> does not exist within the current Guild.\nPlease try setting another channel!",
                        channel
                    )),
            ),
        )
        .await?;
    } else {
        let count = confessions::get_confession_count(
            config.db_url.clone(),
            ctx.guild_id().unwrap().to_string(),
        )
        .await;
        if count.is_err() {
            println!("PLEASEE");
            return Err(count.err().unwrap());
        }
        let count = count.unwrap();
        let message = guild_channel
            .unwrap()
            .send_message(
                &ctx.http(),
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .color(0xFF0000)
                        .title(format!("Confession #{}", count + 1))
                        .description(content.clone()),
                ),
            )
            .await?;
        if let Err(e) = insert_confession(
            config.db_url.clone(),
            message.id.to_string(),
            ctx.author().id.to_string(),
            ctx.guild_id().unwrap().to_string(),
            content,
        )
        .await
        {
            error!("{}", e);
            return Err(Box::from("Could not insert Confession into DB".to_string()));
        }
        ctx.reply(format!("Posted confession here: {}", message.link()))
            .await?;
    }

    Ok(())
}
