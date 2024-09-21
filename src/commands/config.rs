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
            if let Some(delete_vote_min_res) = delete_vote_min {
                changelog.push_str(
                    format!(
                        "Delete Vote Min: {} -> {}\n",
                        guild_config.delete_vote_min, delete_vote_min_res
                    )
                    .as_str(),
                );
                guild_config.delete_vote_min = delete_vote_min_res;
            }
            if let Some(expose_vote_min_res) = expose_vote_min {
                changelog.push_str(
                    format!(
                        "Expose Vote Min: {} :arrow_right: {}\n",
                        guild_config.expose_vote_min, expose_vote_min_res
                    )
                    .as_str(),
                );
                guild_config.expose_vote_min = expose_vote_min_res;
            }
            if let Some(expose_vote_role_res) = expose_vote_role {
                changelog.push_str(
                    format!(
                        "Expose Vote Role: {} :arrow_right: {}\n",
                        guild_config.expose_vote_role.unwrap_or("Unset".to_owned()),
                        expose_vote_role_res
                    )
                    .as_str(),
                );
                guild_config.expose_vote_role = Some(expose_vote_role_res.to_string());
            }
            if let Some(role_ping_res) = role_ping {
                changelog.push_str(
                    format!(
                        "Role Ping: {} :arrow_right: {}\n",
                        guild_config.role_ping.unwrap_or("Unset".to_string()),
                        role_ping_res
                    )
                    .as_str(),
                );
                guild_config.role_ping = Some(role_ping_res.to_string());
            }

            if let Some(guild_id) = ctx.guild_id() {
                guilds::update_guild(
                    config.db_url.clone(),
                    guild_id.to_string(),
                    if let Some(channel_id_res) = channel_id {
                        // TODO: Check if channel is text based
                        changelog.push_str(
                            format!(
                                "Confession Channel ID: {} :arrow_right: {}\n",
                                guild.confession_channel_id.unwrap_or("Unset".to_owned()),
                                channel_id_res
                            )
                            .as_str(),
                        );
                        Some(channel_id_res.to_string())
                    } else {
                        guild.confession_channel_id
                    },
                    guild_config,
                )
                .await?;
            }
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
