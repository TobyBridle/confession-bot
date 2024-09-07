pub mod confess;

pub type Data = ();
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn check(ctx: Context<'_>) -> Result<bool, Error> {
    Ok(!ctx.author().bot)
}
