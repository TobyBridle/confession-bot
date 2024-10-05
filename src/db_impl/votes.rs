use std::error::Error;

use crate::models::{GuildConfig, Vote};
use confession_bot_rs::{establish_connection, schema::delete_votes, VoteType, DELETE_VOTE_STR};
use diesel::{BoolExpressionMethods, ExpressionMethods, IntoSql, QueryDsl, RunQueryDsl};
use ring::digest::{Context, SHA256};
use tracing::error;

use crate::db_impl::{confessions::get_confession_by_message_id, guilds::get_guild};

use super::authors::insert_author;

/// Update the votes for the confession within the DB. If the user has already
/// voted, their vote is removed.
/// # Returns
/// Returns a tuple with members:
///
/// 0 -> The updated amount of votes
///
/// 1 -> The amount of votes required for deletion/exposing
pub async fn update_vote(
    db_url: &String,
    author_id: &String,
    message_id: &String,
    guild_id: &String,
    vote_type: VoteType,
) -> Result<(u32, u32), Box<dyn Error + Send + Sync>> {
    let mut context = Context::new(&SHA256);
    context.update(author_id.as_bytes());
    let hash = format!("{:X?}", context.finish());

    let guild = match get_guild(db_url, guild_id).await? {
        Some(guild) => guild,
        None => {
            return Err(Box::from(format!(
                "Could not find a guild with Guild ID: {}",
                &guild_id
            )))
        }
    };

    let config = match serde_json::from_str::<GuildConfig>(guild.config.as_str()) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Could not parse config {}. Reason: {:?}", guild.config, e);
            return Err(Box::from(e));
        }
    };

    let min_vote = match vote_type {
        VoteType::DELETE => config.delete_vote_min,
        VoteType::EXPOSE => config.expose_vote_min,
    } as u32;

    let vote_type_str: String = vote_type.into();

    let mut connection = establish_connection(db_url);

    let confession = get_confession_by_message_id(db_url, message_id, guild_id).await?;

    // Insert (or, if conflicting, get) the author for the hashed user ID
    let author = insert_author(db_url, &hash).await?;

    let total_votes = match delete_votes::table
        .filter(
            delete_votes::confession_id
                .eq(confession.id)
                .and(delete_votes::vote_type.eq(&vote_type_str)),
        )
        .count()
        .get_result::<i64>(&mut connection)
    {
        Ok(count) => count as u32,
        Err(e) => {
            return Err(Box::from(e));
        }
    };

    // User has already made a vote, so we will remove it
    if let Ok(_) = delete_votes::table
        .filter(
            delete_votes::confession_id
                .eq(confession.id)
                .and(delete_votes::author_id.eq(author))
                .and(delete_votes::vote_type.eq(&vote_type_str)),
        )
        .select(delete_votes::id)
        .first::<i32>(&mut connection)
    {
        // Delete their vote from the DB
        diesel::delete(delete_votes::table)
            .filter(
                delete_votes::confession_id
                    .eq(confession.id)
                    .and(delete_votes::author_id.eq(author))
                    .and(delete_votes::vote_type.eq(&vote_type_str)),
            )
            .execute(&mut connection)?;
        return Ok((total_votes - 1, min_vote));
    }

    if total_votes + 1 == min_vote {
        diesel::delete(delete_votes::table)
            .filter(
                delete_votes::confession_id
                    .eq(confession.id)
                    .and(delete_votes::vote_type.eq(vote_type_str)),
            )
            .execute(&mut connection)?;
        return Ok((min_vote, min_vote));
    }

    match diesel::insert_into(delete_votes::table)
        .values((
            delete_votes::confession_id.eq(confession.id),
            delete_votes::author_id.eq(author),
            delete_votes::vote_type.eq(vote_type_str),
        ))
        .get_result::<Vote>(&mut connection)
    {
        Ok(_) => {
            return Ok((total_votes + 1, min_vote));
        }
        Err(e) => return Err(Box::from(e)),
    }
}
