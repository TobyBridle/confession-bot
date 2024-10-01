use std::error::Error;

use confession_bot_rs::establish_connection;
use diesel::{result, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use tracing::warn;

use crate::{
    models::{Guild, GuildConfig},
    schema::guild::{
        self, confession_channel_id as guildConfessionChannel, config as guildConfig,
        guild_id as guildId,
    },
};

pub async fn get_guild(db_url: &String, guild_id: &String) -> Result<Option<Guild>, result::Error> {
    let mut conn = establish_connection(db_url);
    guild::table
        .filter(guild::guild_id.eq(guild_id))
        .first(&mut conn)
        .optional()
}

pub async fn get_guild_config(
    db_url: &String,
    guild_id: &String,
) -> Result<GuildConfig, Box<dyn Error + Send + Sync>> {
    let mut conn = establish_connection(db_url);
    match guild::table
        .select(guild::config)
        .filter(guild::guild_id.eq(guild_id))
        .first::<String>(&mut conn)
    {
        Ok(cfg) => Ok(serde_json::from_str::<GuildConfig>(cfg.as_str())?),
        Err(e) => Err(Box::from(e)),
    }
}

pub async fn insert_guild(db_url: &String, guild_id: &String) -> Result<(), result::Error> {
    let mut conn = establish_connection(db_url);
    let default_config = GuildConfig {
        delete_vote_min: 10,
        expose_vote_min: 50,
        expose_vote_role: None,
        role_ping: None,
    };
    match serde_json::to_string(&default_config) {
        Ok(default_config_string) => {
            diesel::insert_into(guild::table)
                .values((guildId.eq(guild_id), guildConfig.eq(default_config_string)))
                .on_conflict(guild::guild_id)
                .do_nothing()
                .execute(&mut conn)?;
        }
        Err(e) => {
            warn!("Default guild config not accessible: {}", e)
        }
    }
    Ok(())
}

pub async fn update_guild(
    db_url: &String,
    guild_id: &String,
    confession_channel_id: Option<String>,
    config: GuildConfig,
) -> Result<(), result::Error> {
    let mut conn = establish_connection(db_url);
    match serde_json::to_string(&config) {
        Ok(config_string) => {
            diesel::update(guild::table.filter(guildId.eq(guild_id)))
                .set((
                    guildConfig.eq(config_string),
                    guildConfessionChannel.eq(confession_channel_id),
                ))
                .execute(&mut conn)?;
        }
        Err(e) => {
            warn!("Guild config not accessible: {}", e)
        }
    }
    Ok(())
}
