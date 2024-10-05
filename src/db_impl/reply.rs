use std::error::Error;

use confession_bot_rs::establish_connection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::{db_impl::authors::insert_author, models::Reply, schema::replies};

pub async fn get_confession_replies(
    db_url: &String,
    confession_id: i32,
) -> Result<Vec<Reply>, Box<dyn Error + Send + Sync>> {
    let mut conn = establish_connection(db_url);
    match replies::table
        .select(Reply::as_select())
        .filter(replies::original_confession_id.eq(confession_id))
        .load(&mut conn)
    {
        Ok(replies) => Ok(replies),
        Err(e) => Err(Box::from(e)),
    }
}

pub async fn insert_reply(
    db_url: &String,
    confession_id: i32,
    guild_id: &String,
    message_id: &String,
    content: &String,
    author_id: &String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let author_id = insert_author(db_url, author_id).await?;
    let mut conn = establish_connection(db_url);
    match diesel::insert_into(replies::table)
        .values((
            replies::original_confession_id.eq(confession_id),
            replies::content.eq(content),
            replies::guild_id.eq(guild_id),
            replies::message_id.eq(message_id),
            replies::author.eq(author_id),
        ))
        .execute(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
