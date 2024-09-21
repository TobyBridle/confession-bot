use poise::{
    serenity_prelude::{ChannelId, CreateEmbed, RoleId},
    CreateReply,
};

use crate::{
    commands::{Context, Error},
    db_impl::guilds,
    models::GuildConfig,
};

/// Define a Guild-specific configuration.
#[poise::command(
    slash_command,
    ephemeral,
    rename = "config",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn config_guild(
    ctx: Context<'_>,
    #[description = "The channel to use to post confessions"] channel_id: Option<ChannelId>,
    #[description = "Minimum number of votes required to delete the confession"]
    #[min = 0]
    delete_vote_min: Option<i32>,
    #[description = "Minimum number of votes required to expose the author of the confession"]
    #[min = 0]
    expose_vote_min: Option<i32>,
    #[description = "The minimum role required for the user's vote to count towards exposing the author of the confession"]
    expose_vote_role: Option<RoleId>,
    #[description = "Role to ping when a new Confession is made"] role_ping: Option<RoleId>,
) -> Result<(), Error> {
    let data = ctx.data();
    let config = data.config.read().await;
    if let Some(guild_id) = ctx.guild_id() {
        if let Some(guild) = guilds::get_guild(config.db_url.clone(), guild_id.to_string()).await? {
            let mut changelog = String::new();
            let mut guild_config: GuildConfig = serde_json::from_str(guild.config.as_str())?;
            if delete_vote_min.is_some() {
                changelog.push_str(
                    format!(
                        "Delete Vote Min: {} -> {}\n",
                        guild_config.delete_vote_min,
                        delete_vote_min.unwrap()
                    )
                    .as_str(),
                );
                guild_config.delete_vote_min = delete_vote_min.unwrap();
            }
            if expose_vote_min.is_some() {
                changelog.push_str(
                    format!(
                        "Expose Vote Min: {} :arrow_right: {}\n",
                        guild_config.expose_vote_min,
                        expose_vote_min.unwrap()
                    )
                    .as_str(),
                );
                guild_config.expose_vote_min = expose_vote_min.unwrap();
            }
            if expose_vote_role.is_some() {
                changelog.push_str(
                    format!(
                        "Expose Vote Role: {} :arrow_right: {}\n",
                        guild_config
                            .expose_vote_role
                            .clone()
                            .unwrap_or("Unset".to_owned()),
                        expose_vote_role.unwrap()
                    )
                    .as_str(),
                );
                guild_config.expose_vote_role = Some(expose_vote_role.unwrap().to_string());
            }
            if role_ping.is_some() {
                changelog.push_str(
                    format!(
                        "Role Ping: {} :arrow_right: {}\n",
                        guild_config
                            .role_ping
                            .clone()
                            .unwrap_or(String::from("Unset")),
                        role_ping.unwrap()
                    )
                    .as_str(),
                );
                guild_config.role_ping = Some(role_ping.unwrap().to_string());
            }

            guilds::update_guild(
                config.db_url.clone(),
                ctx.guild_id().unwrap().to_string(),
                if channel_id.is_some() {
                    // TODO: Check if channel is text based
                    changelog.push_str(
                        format!(
                            "Confession Channel ID: {} :arrow_right: {}\n",
                            guild
                                .confession_channel_id
                                .clone()
                                .unwrap_or(String::from("Unset")),
                            channel_id.unwrap()
                        )
                        .as_str(),
                    );
                    Some(channel_id.unwrap().to_string())
                } else {
                    guild.confession_channel_id
                },
                guild_config,
            )
            .await?;
            ctx.send(
                CreateReply::default().embed(
                    CreateEmbed::default()
                        .title("Configuration Changes")
                        .color(0x00FF00)
                        .description(changelog),
                ),
            )
            .await?;
        }
    }
    Ok(())
}
