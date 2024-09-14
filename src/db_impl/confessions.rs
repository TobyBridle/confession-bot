use std::error::Error;

use confession_bot_rs::establish_connection;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::{
    models::Confession,
    schema::{
        confession,
        guild::{self},
    },
};

pub async fn get_confession_by_id(
    db_url: String,
    message_id: String,
    _guild_id: String,
) -> Result<Confession, Box<dyn Error>> {
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
        Ok(c) => return Ok(c.clone()),
        Err(e) => return Err(Box::new(e)),
    }
}
