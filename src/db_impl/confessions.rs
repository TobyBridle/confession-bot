use std::error::Error;

use confession_bot_rs::establish_connection;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::{
    db_impl::authors::insert_author,
    models::Confession,
    schema::{
        confession,
        guild::{self},
    },
};

pub async fn get_confession_by_id_guild(
    db_url: &String,
    confession_id: u32,
    guild_id: &String,
) -> Result<Confession, Box<dyn Error + Send + Sync>> {
    let mut connection = establish_connection(db_url);
    match guild::table
        .inner_join(confession::table)
        .limit(1)
        .offset((confession_id - 1).into())
        .select(Confession::as_select())
        .get_result(&mut connection)
    {
        Ok(c) => Ok(c.clone()),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn get_confession_by_message_id(
    db_url: &String,
    message_id: &String,
    _guild_id: &String,
) -> Result<Confession, Box<dyn Error + Send + Sync>> {
    let mut connection = establish_connection(db_url);
    match guild::table
        .inner_join(confession::table)
        .filter(
            confession::message_id
                .eq(message_id)
                .and(guild::guild_id.eq(_guild_id)),
        )
        .select(Confession::as_select())
        .get_result(&mut connection)
    {
        Ok(c) => Ok(c.clone()),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn get_confession_count(
    db_url: &String,
    _guild_id: &String,
) -> Result<i64, Box<dyn Error + Send + Sync>> {
    use self::confession::dsl::*;
    let mut connection = establish_connection(db_url);
    let c = confession
        .inner_join(guild::table)
        .select(Confession::as_select())
        .get_results(&mut connection);
    match c {
        Ok(c) => Ok(c.len() as i64),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn insert_confession(
    db_url: &String,
    message_id: &String,
    _author_id: &String,
    _guild_id: &String,
    content: &String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let author_id = insert_author(db_url, _author_id).await?;
    let mut conn = establish_connection(db_url);
    match diesel::insert_into(confession::table)
        .values((
            confession::content.eq(content),
            confession::guild_id.eq(_guild_id),
            confession::message_id.eq(message_id),
            confession::author.eq(author_id),
        ))
        .execute(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
