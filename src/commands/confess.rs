use std::str::FromStr;

use confession_bot_rs::{DELETE_VOTE_STR, EXPOSE_VOTE_STR};
use poise::{
    serenity_prelude::{
        ButtonStyle, Channel, ChannelId, CreateActionRow, CreateButton, CreateEmbed, CreateMessage,
        ReactionType,
    },
    CreateReply,
};
use rand::random;
use tracing::{error, warn};

use crate::db_impl::guilds;
use crate::{
    commands::{Context, Error},
    db_impl::confessions::{self, insert_confession},
    models::GuildConfig,
};

/// Post a confession into the confession channel
#[poise::command(slash_command, ephemeral, rename = "confess")]
pub async fn confession(
    ctx: Context<'_>,
    #[description = "The confession text content"] content: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let config = data.config.read().await;
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let guild = match guilds::get_guild(&config.db_url, &guild_id.to_string()).await? {
        Some(guild) => guild,
        None => {
            ctx.send(CreateReply::default().embed(
                CreateEmbed::default()
                    .color(0xFF0000)
                    .title("Database Error")
                    .description("The current Guild does not exist within the Bot's Database. Please attempt to run the command again.")
            ))
            .await?;
            guilds::insert_guild(&config.db_url, &guild_id.to_string()).await?;
            return Ok(());
        }
    };

    let guild_config = match serde_json::from_str::<GuildConfig>(guild.config.as_str()) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!(
                "Could not convert {} to guild config. Reason: {:?}",
                guild.config, e
            );
            return Err(Box::from(e));
        }
    };

    let channel_id = match guild.confession_channel_id {
        Some(id) => id,
        None => {
            ctx.send(
                CreateReply::default().embed(
                    CreateEmbed::default()
                        .title("Guild Error")
                        .description("A confession channel must be set before using `/confess`!\nPlease request a moderator to set one using the `/config` command.")
                        .color(0xFFAA00)
                )
            )
            .await?;
            return Ok(());
        }
    };

    let guild_channel = ctx
        .http()
        .get_channel(ChannelId::new(channel_id.parse()?))
        .await?;

    let guild_channel = match guild_channel {
        Channel::Guild(channel) => channel,
        _ => {
            ctx.send(
                CreateReply::default().embed(
                    CreateEmbed::default()
                        .color(0xFF0000)
                        .title("Guild Error")
                        .description(format!(
                            "Channel <#{}> is not a valid guild channel.\nPlease try setting another channel!",
                            channel_id
                        )),
                ),
            )
            .await?;
            return Ok(());
        }
    };

    let count = match confessions::get_confession_count(&config.db_url, &guild_id.to_string()).await
    {
        Ok(count) => count,
        Err(e) => {
            warn!("Failed to get confession count: {}", e);
            return Err(e);
        }
    };

    let message_res = guild_channel
        .send_message(
            ctx.http(),
            CreateMessage::default()
                .embed(
                    CreateEmbed::default()
                        .color(random::<u16>() as u32)
                        .title(format!("Confession #{}", count + 1))
                        .description(content.clone()),
                )
                .components(&[CreateActionRow::Buttons(vec![
                    CreateButton::new(DELETE_VOTE_STR)
                        .emoji(ReactionType::from_str("🗑")?)
                        .style(ButtonStyle::Danger)
                        .label(format!("Delete (0/{})", guild_config.delete_vote_min)),
                    CreateButton::new(EXPOSE_VOTE_STR)
                        .emoji(ReactionType::from_str("🕵️")?)
                        .label(format!("Expose (0/{})", guild_config.expose_vote_min)),
                ])]),
        )
        .await;

    match message_res {
        Ok(message) => {
            if let Err(e) = insert_confession(
                &config.db_url,
                &message.id.to_string(),
                &ctx.author().id.to_string(),
                &guild_id.to_string(),
                &content,
            )
            .await
            {
                // TODO: Delete the confession as we are unable to moderate and accept votes
                // if it is not within the DB
                error!("{}", e);
                return Err(Box::from("Could not insert Confession into DB".to_owned()));
            }

            ctx.reply(format!("Posted confession here: {}", message.link()))
                .await?;
        }
        Err(e) => {
            error!("Could not post confession: {}", e);
            return Err(Box::from(format!(
                "Could not send confession. Reason: {}",
                e.to_string()
            )));
        }
    }

    Ok(())
}
