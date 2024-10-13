use chrono::{Duration, Local, Utc};
use confession_bot_rs::establish_connection;
use diesel::{ExpressionMethods, RunQueryDsl, SelectableHelper, SqliteConnection};
use poise::{
    serenity_prelude::{Timestamp, UserId},
    ChoiceParameter,
};

use crate::{
    commands::{Context, Error},
    models::{InsertSchedule, Schedule},
    schema::schedule,
};

#[derive(Debug, ChoiceParameter, Copy, Clone)]
enum TimeoutDuration {
    #[name = "60 seconds"]
    SECONDS_60,
    #[name = "5 minutes"]
    MINS_5,
    #[name = "10 Minutes"]
    MINS_10,
    #[name = "1 Hour"]
    HOURS_1,
    #[name = "1 Day"]
    DAYS_1,
    #[name = "1 Week"]
    WEEKS_1,
}

impl From<TimeoutDuration> for i64 {
    fn from(value: TimeoutDuration) -> Self {
        match value {
            TimeoutDuration::SECONDS_60 => 60,
            TimeoutDuration::MINS_5 => 300,
            TimeoutDuration::MINS_10 => 600,
            TimeoutDuration::HOURS_1 => 3600,
            TimeoutDuration::DAYS_1 => 3600 * 24,
            TimeoutDuration::WEEKS_1 => 3600 * 24 * 7,
        }
    }
}

impl From<TimeoutDuration> for &str {
    fn from(value: TimeoutDuration) -> Self {
        match value {
            TimeoutDuration::SECONDS_60 => "60 seconds",
            TimeoutDuration::MINS_5 => "5 minutes",
            TimeoutDuration::MINS_10 => "10 minutes",
            TimeoutDuration::HOURS_1 => "1 hour",
            TimeoutDuration::DAYS_1 => "1 day",
            TimeoutDuration::WEEKS_1 => "1 week",
        }
    }
}

#[poise::command(slash_command, ephemeral, rename = "schedule", owners_only)]
pub async fn schedule_timeout(
    ctx: Context<'_>,
    #[description = "The user to timeout"] victim: UserId,
    #[description = "In how long the timeout should begin"] start_in: String,
    #[description = "How long the timeout should last"] ends_in: TimeoutDuration,
) -> Result<(), Error> {
    let now = Utc::now();
    let data = ctx.data();
    let config = data.config.read().await;

    // Check if the string ends with 'h' or 'm'
    let unit = start_in.chars().last().ok_or("Empty string")?;

    // Parse the number part of the string
    let number_part = &start_in[..start_in.len() - 1];
    let value: i64 = number_part.parse().map_err(|_| "Invalid number")?;

    // Calculate the new time based on the unit (hours or minutes)
    let start_time = match unit {
        's' => now + Duration::seconds(value),
        'h' => now + Duration::hours(value),
        'm' => now + Duration::minutes(value),
        'd' => now + Duration::days(value),
        _ => return Err("Invalid time unit".into()),
    };

    match victim {
        victim if victim == ctx.author().id => {
            return Err(Box::from("You cannot schedule a timeout on yourself!"));
        }
        victim if victim == ctx.framework().bot_id() => {
            return Err(Box::from("You cannot schedule a timeout on the bot!"));
        }
        _ => {
            let mut connection = establish_connection(&config.db_url);
            let insert_schedule = InsertSchedule {
                victim_id: victim.to_string(),
                guild_id: ctx.guild_id().unwrap().to_string(),
                ends_at: i32::try_from(Into::<i64>::into(ends_in) + start_time.timestamp())
                    .unwrap(),
                start_at: i32::try_from(start_time.timestamp()).unwrap(),
            };
            match diesel::insert_into(schedule::table)
                .values(&insert_schedule)
                .returning(Schedule::as_returning())
                .get_result(&mut connection)
            {
                Ok(_) => {
                    ctx.reply(
                        format!("Succesfully scheduled a timeout for User <@{}> starting <t:{}:R> lasting for {}", victim.to_string(), start_time.timestamp(), Into::<&str>::into(ends_in))
                    ).await?;
                    Ok(())
                }
                Err(e) => return Err(Box::from(e)),
            }
        }
    }
}
