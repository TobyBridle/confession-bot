use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

use diesel::{
    associations::HasTable, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper,
    SqliteConnection,
};

use crate::{
    models::Confession,
    schema::{confession, guild},
};

pub async fn get_confession_by_id(
    connection: &mut SqliteConnection,
    confession_id: String,
    guild_id: String,
) -> Result<Confession, Box<dyn Error>> {
    use self::confession::dsl::*;
    match confession
        .inner_join(guild::table)
        .select(Confession::as_select())
        .load::<Confession>(connection)
    {
        Ok(c) => {
            return if let Some(c) = c.first() {
                Ok(c.clone())
            } else {
                Err("Test".into())
            }
        }
        Err(e) => return Err(Box::new(e)),
    }
}
