use crate::{
    commands::{Context, Error},
    db_impl::guilds,
};

#[poise::command(slash_command, ephemeral, rename = "confess")]
pub async fn confession(
    ctx: Context<'_>,
    #[description = "The confession text content"] content: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let config = data.config.read().await;
    let guild =
        guilds::get_guild(config.db_url.clone(), ctx.guild_id().unwrap().to_string()).await?;
    println!("{:?}", guild);
    Ok(())
}
