use std::{error::Error, io::Read};

use confession_bot_rs::establish_connection;
use diesel::{ExpressionMethods, RunQueryDsl};
use ring::digest::{Context, SHA256};

use crate::schema::authors;

pub async fn insert_author(db_url: String, author_id: String) -> Result<i32, Box<dyn Error>> {
    let mut connection = establish_connection(db_url);
    let mut context = Context::new(&SHA256);
    context.update(&author_id.as_bytes());
    let mut hash = String::new();
    let _hash = context.finish().as_ref().read_to_string(&mut hash);

    match diesel::insert_into(authors::table)
        .values(authors::hash.eq(hash))
        .returning(authors::id)
        .get_result::<i32>(&mut connection)
    {
        Ok(u) => Ok(u),
        Err(e) => Err(Box::new(e)),
    }
}
