use std::error::Error;

use confession_bot_rs::establish_connection;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use ring::digest::{Context, SHA256};

use crate::schema::authors;

pub async fn get_author_by_hash(db_url: String, hash: String) -> Result<i32, Box<dyn Error>> {
    let mut connection = establish_connection(db_url);
    match authors::table
        .select(authors::hash)
        .filter(authors::hash.eq(hash))
        .execute(&mut connection)
    {
        Ok(id) => Ok(id as i32),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn insert_author(db_url: String, author_id: String) -> Result<i32, Box<dyn Error>> {
    let mut connection = establish_connection(db_url.clone());
    let mut context = Context::new(&SHA256);
    context.update(&author_id.as_bytes());
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
