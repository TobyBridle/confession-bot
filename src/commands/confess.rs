use crate::commands::*;

#[poise::command(slash_command, ephemeral, check = "check", rename = "confess")]
pub async fn confession(
    ctx: Context<'_>,
    #[description = "The confession text content"] content: String,
) -> Result<(), Error> {
    ctx.reply("some shi").await?;
    Ok(())
}
