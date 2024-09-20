use confession_bot_rs::establish_connection;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

use crate::{
    models::{Guild, GuildConfig},
    schema::guild::{
        self, confession_channel_id as guildConfessionChannel, config as guildConfig,
        guild_id as guildId,
    },
};

pub async fn get_guild(
    db_url: String,
    guild_id: String,
) -> Result<Option<Guild>, diesel::result::Error> {
    let mut conn = establish_connection(db_url);
    guild::table
        .filter(guild::guild_id.eq(guild_id))
        .first(&mut conn)
        .optional()
}

pub async fn insert_guild(db_url: String, guild_id: String) -> Result<(), diesel::result::Error> {
    let mut conn = establish_connection(db_url);
    let default_config = GuildConfig {
        delete_vote_min: 10,
        expose_vote_min: 50,
        expose_vote_role: None,
        role_ping: None,
    };
    diesel::insert_into(guild::table)
        .values((
            guildId.eq(guild_id),
            guildConfig.eq(serde_json::to_string(&default_config).unwrap()),
        ))
        .execute(&mut conn)?;
    Ok(())
}

pub async fn update_guild(
    db_url: String,
    guild_id: String,
    confession_channel_id: Option<String>,
    config: GuildConfig,
) -> Result<(), diesel::result::Error> {
    let mut conn = establish_connection(db_url);
    diesel::update(guild::table.filter(guildId.eq(guild_id)))
        .set((
            guildConfig.eq(serde_json::to_string(&config).unwrap()),
            guildConfessionChannel.eq(confession_channel_id),
        ))
        .execute(&mut conn)?;
    Ok(())
}
