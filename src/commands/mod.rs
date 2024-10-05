use std::{str::FromStr, sync::Arc};

use confession_bot_rs::{VoteType, DELETE_VOTE_STR, EXPOSE_VOTE_STR};
use poise::{
    builtins,
    serenity_prelude::{
        self as serenity, ActionRowComponent, ButtonStyle, CreateActionRow, CreateButton,
        CreateEmbed, CreateEmbedFooter, CreateMessage, EditMessage, GuildId, ReactionType, RoleId,
        UserId,
    },
    FrameworkContext, FrameworkError,
};
use ring::digest::SHA256;
use serenity::FullEvent;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::{
    db_impl::{
        authors::get_author_hash_by_message,
        guilds::{self, get_guild_config},
        votes::update_vote,
    },
    models::GuildConfig,
    Config,
};

pub mod confess;
pub mod config;
pub mod reply;

pub struct Data {
    pub config: RwLock<Config>,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn event_handler(
    framework: FrameworkContext<'_, Data, Error>,
    event: &FullEvent,
) -> Result<(), Error> {
    match &event {
        FullEvent::InteractionCreate { interaction } => {
            if let Some(cmp) = interaction.as_message_component() {
                let reaction_type: VoteType = cmp.data.custom_id.to_string().into();
                let message_id = cmp.message.id.to_string();
                let author_id = cmp.user.id.to_string();
                let guild_id = cmp
                    .guild_id
                    .ok_or("Could not get Guild ID for interaction.");
                if let Err(e) = guild_id {
                    return Err(Box::from(e.to_string()));
                }
                let guild_id = match guild_id {
                    Ok(id) => id.to_string(),
                    Err(e) => {
                        error!("Could not get guild id. Reason: {:?}", e);
                        return Err(Box::from(e));
                    }
                };

                let data: Arc<Data> = framework.serenity_context.data();
                let config = data.config.read().await;
                let guild_config = get_guild_config(&config.db_url, &guild_id).await?;

                if reaction_type == VoteType::DELETE {
                    let updated = update_vote(
                        &config.db_url,
                        &author_id,
                        &message_id,
                        &guild_id,
                        reaction_type,
                    )
                    .await?;

                    let updated_message = if updated.0 == updated.1 {
                        EditMessage::new()
                            .embed(
                                CreateEmbed::new()
                                    .title(
                                        cmp.message
                                            .embeds
                                            .first()
                                            .unwrap()
                                            .title
                                            .as_ref()
                                            .unwrap()
                                            .to_string(),
                                    )
                                    .description(format!(
                                        "Deleted Confession ({} votes)",
                                        updated.0
                                    ))
                                    .color(0xFF0000),
                            )
                            .components(vec![])
                    } else {
                        let action_row = match cmp.message.components.first() {
                            Some(c) => c,
                            None => {
                                return Err(Box::from("Could not get components from message."))
                            }
                        };

                        let expose = match action_row.components.iter().nth(1) {
                            Some(ActionRowComponent::Button(b)) => b,
                            e => {
                                panic!("Did not find the right component! Got: {:?}", e)
                            }
                        };
                        let components = CreateActionRow::Buttons(vec![
                            CreateButton::new(DELETE_VOTE_STR)
                                .emoji(ReactionType::from_str("🗑")?)
                                .style(ButtonStyle::Danger)
                                .label(format!("Delete ({}/{})", updated.0, updated.1)),
                            expose.clone().into(),
                        ]);
                        EditMessage::new().components(vec![components])
                    };

                    cmp.message
                        .clone()
                        .edit(&framework.serenity_context.http, updated_message)
                        .await?;
                } else {
                    if let Some(minimum_role) = guild_config.expose_vote_role {
                        if !cmp
                            .user
                            .has_role(
                                &framework.serenity_context.http,
                                cmp.guild_id.unwrap_or(GuildId::default()),
                                RoleId::from_str(&minimum_role)?,
                            )
                            .await?
                        {
                            return Ok(());
                        }
                    }
                    let updated = update_vote(
                        &config.db_url,
                        &author_id,
                        &message_id,
                        &guild_id,
                        reaction_type,
                    )
                    .await?;

                    let updated_message = if updated.0 == updated.1 {
                        // Attempt to find the author
                        let guild = framework
                            .serenity_context
                            .http
                            .get_guild(cmp.guild_id.unwrap())
                            .await?;
                        let author_hash = get_author_hash_by_message(
                            &config.db_url,
                            &cmp.message.id.to_string(),
                            &guild_id,
                        )
                        .await?;
                        let mut first_author_id: Option<UserId> = None;
                        let mut last_user_id: Option<UserId> = None;

                        let mut author =
                            "Unknown (Author may no longer be within the Guild)".to_string();

                        for _ in (0..250_000).skip(1000) {
                            let members = guild
                                .members(&framework.serenity_context.http, None, last_user_id)
                                .await?;

                            if let Some(member) = members.first() {
                                if Some(member.user.id) == first_author_id {
                                    break;
                                }
                                if first_author_id.is_none() {
                                    first_author_id = Some(member.user.id);
                                }
                            }

                            let found = members.iter().enumerate().find(|(i, m)| {
                                let mut context = ring::digest::Context::new(&SHA256);
                                context.update(m.user.id.to_string().as_bytes());

                                let hash = format!("{:X?}", context.finish());

                                if i % 1000 == 0 {
                                    last_user_id = Some(m.user.id)
                                } else if *i == members.len() - 1 {
                                    last_user_id = None;
                                }

                                return hash == author_hash;
                            });

                            if let Some(f) = found {
                                author = format!(
                                    "{} - ({})",
                                    f.1.user.display_name(),
                                    f.1.user.id.to_string()
                                );
                                break;
                            }
                        }
                        EditMessage::new()
                            .embed(
                                CreateEmbed::from(cmp.message.embeds.first().unwrap().clone())
                                    .title(format!(
                                        "Exposed {}",
                                        cmp.message
                                            .embeds
                                            .first()
                                            .unwrap()
                                            .title
                                            .as_ref()
                                            .unwrap()
                                            .to_string()
                                    ))
                                    .footer(CreateEmbedFooter::new(format!("Author: {}", author))),
                            )
                            .components(vec![])
                    } else {
                        let action_row = match cmp.message.components.first() {
                            Some(c) => c,
                            None => {
                                return Err(Box::from("Could not get components from message."))
                            }
                        };

                        let delete = match action_row.components.iter().nth(0) {
                            Some(ActionRowComponent::Button(b)) => b,
                            e => {
                                panic!("Did not find the right component! Got: {:?}", e)
                            }
                        };
                        let components = CreateActionRow::Buttons(vec![
                            delete.clone().into(),
                            CreateButton::new(EXPOSE_VOTE_STR)
                                .emoji(ReactionType::from_str("🕵️")?)
                                .label(format!("Expose ({}/{})", updated.0, updated.1)),
                        ]);
                        EditMessage::new().components(vec![components])
                    };

                    cmp.message
                        .clone()
                        .edit(&framework.serenity_context.http, updated_message)
                        .await?;
                }
            }
        }
        FullEvent::Ready { data_about_bot } => {
            info!(
                "Logged in as: {}. Currently observing {} guild(s)",
                data_about_bot.user.name,
                data_about_bot.guilds.len()
            );
            builtins::register_globally(
                &framework.serenity_context.http,
                &framework.options().commands,
            )
            .await?;
        }
        FullEvent::GuildCreate {
            guild,
            is_new: Some(new_guild),
        } => {
            if *new_guild {
                let data = framework.serenity_context.data::<Data>();
                let config = data.config.read().await;
                guilds::insert_guild(&config.db_url, &guild.id.to_string()).await?;
                info!("Joined Guild {}", guild.id)
            }
        }
        FullEvent::Message { new_message } => {
            if new_message.mentions_user_id(framework.bot_id()) {
                let data = framework.serenity_context.data::<Data>();
                let config = data.config.read().await;
                if let Some(guild_id) = new_message.guild_id {
                    let guild = {
                        if let Some(g) =
                            guilds::get_guild(&config.db_url, &guild_id.to_string()).await?
                        {
                            g
                        } else {
                            error!(
                                "Could not get guild ({}) from DB! Please check that the database exists!",
                                guild_id
                            );
                            return Err(Box::from("Guild not found".to_owned()));
                        }
                    };
                    if let Ok(config) = serde_json::from_str::<GuildConfig>(&guild.config) {
                        let embed = CreateEmbed::default()
                            .title("Guild Configuration")
                            .description("It is recommended that you use a designated confession channel in order to prevent 'spam' of an already used text-channel.")
                            .fields(
                                vec![
                                    ("Confession Channel",
                                    if let Some(confession_channel) = guild.confession_channel_id {
                                        format!("<#{}>", confession_channel)
                                    } else {
                                        "Unset".to_owned()
                                    }, true),
                                    ("", "".to_owned(), true),
                                    ("", "".to_owned(), true),
                                    ("Minimum Vote (Delete)", config.delete_vote_min.to_string(), true),
                                    ("", "".to_owned(), true),
                                    ("Minimum Vote (Expose)", config.expose_vote_min.to_string(), true),
                                    ("Minimum Role to register  (Expose)",
                                    if let Some(expose_vote_role) = config.expose_vote_role {
                                        format!("<@{}>", expose_vote_role)
                                    } else {
                                        "Unset".to_string()
                                    }, true),
                                    ("", "".to_owned(), true),
                                    ("Role Ping",
                                    if let Some(ping_role) = config.role_ping {
                                        format!("<@{}>", ping_role)
                                    } else {
                                        "Unset".to_owned()
                                    }, true)
                                ]
                            )
                            .color(0x11FF00);
                        if let Ok(guild_channel) = new_message
                            .guild_channel(&framework.serenity_context.http)
                            .await
                        {
                            guild_channel
                                .send_message(
                                    &framework.serenity_context.http,
                                    CreateMessage::default().add_embeds([embed]),
                                )
                                .await?;
                        }
                    }
                }
            }
            return Ok(());
        }
        _ => {}
    }
    Ok(())
}

pub async fn on_error(err: FrameworkError<'_, Data, Error>) {
    match err {
        FrameworkError::UnknownCommand { framework, msg, .. } => {
            if msg.mentions_user_id(framework.bot_id()) {}
        }
        err => {
            if let Err(e) = builtins::on_error(err).await {
                error!("Error while handling error: {:?}", e);
            }
        }
    }
}
