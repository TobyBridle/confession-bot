use std::{str::FromStr, sync::Arc};

use confession_bot_rs::{VoteType, DELETE_VOTE_STR};
use poise::{
    builtins,
    serenity_prelude::{
        self as serenity, ActionRowComponent, ButtonStyle, CreateActionRow, CreateButton,
        CreateEmbed, CreateMessage, EditMessage, ReactionType,
    },
    FrameworkContext, FrameworkError,
};
use serenity::FullEvent;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::{
    db_impl::{guilds, votes::update_vote},
    models::GuildConfig,
    Config,
};

pub mod confess;
pub mod config;

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
                let guild_id = guild_id.unwrap().to_string();

                let data: Arc<Data> = framework.serenity_context.data();
                let config = data.config.read().await;

                if reaction_type == VoteType::DELETE {
                    let updated = update_vote(
                        config.db_url.clone(),
                        author_id,
                        message_id,
                        guild_id,
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
                    panic!("Not Implemented Reaction type for Expose")
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
                guilds::insert_guild(config.db_url.clone(), guild.id.to_string()).await?;
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
                            guilds::get_guild(config.db_url.clone(), guild_id.to_string()).await?
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
