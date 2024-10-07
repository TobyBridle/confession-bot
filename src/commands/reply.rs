use std::collections::HashMap;

use crate::{
    commands::{Context, Error},
    db_impl::{
        confessions::get_confession_by_id_guild,
        guilds::get_guild,
        reply::{get_confession_replies, insert_reply},
    },
};
use poise::serenity_prelude::{ChannelId, CreateEmbed, CreateMessage, GuildChannel, MessageId};
use rand::random;
use tracing::error;

#[poise::command(slash_command, ephemeral)]
pub async fn reply(
    ctx: Context<'_>,
    #[description = "The ID of the confession to respond to"] id: u32,
    #[description = "The confession text content"] content: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let config = data.config.read().await;

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            error!("Could not get Guild ID whilst replying.");
            return Err(Box::from("Could not get Guild ID whilst replying."));
        }
    };

    let confession_channel =
        match get_guild(&config.db_url, &guild_id.to_string()).await {
            Ok(res) => {
                if let Some(g) = res {
                    match g.confession_channel_id {
                        Some(id) => id,
                        None => return Err(Box::from(
                            "Could not find a confession channel. Are you sure it has been set?",
                        )),
                    }
                } else {
                    return Err(Box::from(
                        "Could not find guild within the database. Try making a confession first.",
                    ));
                }
            }
            Err(e) => {
                return Err(Box::from(e));
            }
        };

    if let Ok(confession) =
        get_confession_by_id_guild(&config.db_url, id, &guild_id.to_string()).await
    {
        // Check that the message hasn't been deleted.
        if confession.deleted == 1 {
            ctx.reply("Cannot respond to the Confession. Reason: Confession has been deleted.")
                .await?;
            return Ok(());
        }

        let mut map = HashMap::new();
        map.insert("name", format!("Confession {} replies", id));
        let reply_channel: GuildChannel = if get_confession_replies(&config.db_url, confession.id)
            .await?
            .len()
            == 0
        {
            match ctx
                .http()
                .create_thread_from_message(
                    ChannelId::new(confession_channel.parse().unwrap()),
                    MessageId::new(confession.message_id.parse().unwrap()),
                    &map,
                    None,
                )
                .await
            {
                Ok(res) => res,
                Err(e) => return Err(Box::from(e)),
            }
        } else {
            let threads = match ctx.http().get_guild_active_threads(guild_id).await {
                Ok(t) => t.threads,
                Err(e) => {
                    return Err(Box::from(format!("Could not find the original confession to respond to. The thread may no longer be active.\n\nReason: {:?}", e)));
                }
            };

            let channel = threads
                .iter()
                .find(|t| t.id.to_string() == confession.message_id);
            match channel.cloned() {
                Some(c) => c,
                None => return Err(Box::from("Could not find the thread to respond to.")),
            }
        };

        let message_res = match reply_channel
            .send_message(
                ctx.http(),
                CreateMessage::default().embed(
                    CreateEmbed::default()
                        .color(random::<u16>() as u32)
                        .title(format!("Response to Confession"))
                        .description(content.clone()),
                ),
            )
            .await
        {
            Ok(m) => m,
            Err(e) => {
                return Err(Box::from(e));
            }
        };

        insert_reply(
            &config.db_url,
            confession.id,
            &confession.guild_id,
            &message_res.id.to_string(),
            &content,
            &ctx.author().id.to_string(),
        )
        .await?;
    } else {
        return Err(Box::from(format!(
            "Could not find confession with ID `{}` in the Guild.",
            id
        )));
    }
    ctx.reply(format!("Successfully replied to Confession {}", id))
        .await?;
    Ok(())
}
