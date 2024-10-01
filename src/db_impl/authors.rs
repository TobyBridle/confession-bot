use std::error::Error;

use confession_bot_rs::establish_connection;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use ring::digest::{Context, SHA256};
use tracing::error;

use crate::schema::{authors, confession};

pub async fn get_author_by_hash(
    db_url: &String,
    hash: String,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let mut connection = establish_connection(&db_url);
    match authors::table
        .select(authors::id)
        .filter(authors::hash.eq(hash))
        .first::<i32>(&mut connection)
    {
        Ok(id) => Ok(id),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn get_author_hash_by_message(
    db_url: &String,
    message_id: &String,
    guild_id: &String,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut connection = establish_connection(&db_url);
    let author_id = match confession::table
        .select(confession::author)
        .filter(
            confession::message_id
                .eq(&message_id)
                .and(confession::guild_id.eq(&guild_id)),
        )
        .first::<i32>(&mut connection)
    {
        Ok(id) => id,
        Err(e) => {
            error!(
                "Could not find author id in DB using {} and {}",
                &message_id, &guild_id
            );
            return Err(Box::from(e));
        }
    };

    match authors::table
        .select(authors::hash)
        .filter(authors::id.eq(author_id))
        .first::<String>(&mut connection)
    {
        Ok(hash) => Ok(hash),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn insert_author(
    db_url: &String,
    author_id: &String,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let mut connection = establish_connection(db_url);
    let mut context = Context::new(&SHA256);
    context.update(author_id.as_bytes());
    let hash = format!("{:X?}", context.finish());

    match diesel::insert_into(authors::table)
        .values(authors::hash.eq(hash.clone()))
        .on_conflict(authors::hash)
        .do_nothing()
        .returning(authors::id)
        .get_result::<i32>(&mut connection)
        .optional()
    {
        Ok(u) => {
            if let Some(id) = u {
                Ok(id)
            } else {
                // We have had a conflict, and can access the ID using the hash
                Ok(get_author_by_hash(db_url, hash).await?)
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}
