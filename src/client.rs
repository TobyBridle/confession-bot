use chrono::Utc;
use confession_bot_rs::establish_connection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use poise::serenity_prelude::{
    self, Client, GatewayIntents, Guild, Http, Settings, Timestamp, UserId,
};
use poise::{Framework, FrameworkContext, FrameworkOptions};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::commands::Error;
use crate::models::Schedule;
use crate::{commands::*, Config};

pub async fn start(config: Config) -> anyhow::Result<()> {
    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![
                confess::confession(),
                reply::reply(),
                config::config_guild(),
                schedule::schedule_timeout(),
            ],
            event_handler: |ctx, event| Box::pin(event_handler(ctx, event)),
            on_error: |err| Box::pin(on_error(err)),
            ..Default::default()
        })
        .build();
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let cache_settings = Settings::default();
    let client = Client::builder(config.bot_token.as_str(), intents)
        .framework(framework)
        .cache_settings(cache_settings)
        .data(Arc::new(Data {
            config: RwLock::new(config),
        }))
        .await;
    match client {
        Ok(mut client) => {
            if let Err(e) = client.start().await {
                warn!("Client error: {:?}", e);
                panic!();
            }
        }
        Err(e) => {
            warn!("Error creating client: {:?}", e);
            panic!();
        }
    }

    Ok(())
}

pub async fn observe(ctx: FrameworkContext<'_, Data, Error>, guild: Guild) {
    let data = ctx.user_data();
    let config = &data.config;
    let db_url = config.read().await.db_url.clone();

    loop {
        // Clone the necessary parts of ctx and guild to avoid lifetime issues
        let serenity_http = ctx.serenity_context.http.clone();
        let guild_clone = guild.clone();
        let db_url_clone = db_url.clone();

        // Spawn an asynchronous task to handle the database checking and communication disabling
        tokio::spawn(async move {
            if let Err(e) = process_schedules(serenity_http, &guild_clone, &db_url_clone).await {
                eprintln!("Error processing schedules: {:?}", e);
            }
        });

        // Wait for 30 seconds before repeating the process
        sleep(Duration::from_secs(30)).await;
    }
}

// Async function to process schedules and disable communication for members
async fn process_schedules(
    serenity_http: Arc<Http>,
    guild: &Guild,
    db_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Offload the database querying to a blocking thread
    let schedules = tokio::task::spawn_blocking({
        let db_url = db_url.to_string();
        move || -> Result<Vec<Schedule>, diesel::result::Error> {
            let mut connection = establish_connection(&db_url);
            crate::schema::schedule::table
                .select(Schedule::as_select())
                .load::<Schedule>(&mut connection)
        }
    })
    .await??; // Await the task and propagate any errors

    for schedule in schedules {
        if guild.id.to_string() != schedule.guild_id {
            continue;
        }
        if schedule.start_at as i64 <= Utc::now().timestamp() {
            info!(
                "Disabling communication for member with ID: {}",
                schedule.victim_id
            );
            let res =
                disable_communication_for_member(serenity_http.clone(), guild, schedule.clone())
                    .await;
            match res {
                Ok(_) => {
                    info!(
                        "Removing scheduled timeout from DB for ID: {}",
                        schedule.victim_id
                    );
                    let _ = delete_schedule(schedule.id, db_url).await.unwrap();
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

// Function to disable communication for a guild member
async fn disable_communication_for_member(
    serenity_http: Arc<Http>,
    guild: &Guild,
    schedule: Schedule,
) -> Result<(), Error> {
    // Only attempt in the correct guild
    if guild.id.to_string() != schedule.guild_id {
        return Err(Box::from(format!(
            "Cannot disable user {} in guild {}. Reason: Schedule was set for guild {}.",
            schedule.victim_id,
            guild.id.to_string(),
            schedule.guild_id
        )));
    }

    let victim_id = schedule.victim_id.parse::<u64>()?;
    let ends_at = Timestamp::from_unix_timestamp(schedule.ends_at as i64)?;

    // Retrieve the member, and ensure it is owned by calling `into_owned()`
    let mut member = guild
        .member(&serenity_http, UserId::new(victim_id))
        .await?
        .into_owned(); // This converts the `Cow` into an owned `Member`

    // Now that we have an owned `Member`, we can safely modify it
    member
        .disable_communication_until(&serenity_http, ends_at)
        .await?;

    println!("Disabled communication for member with ID: {}", victim_id);
    Ok(())
}

// Function to delete the schedule from the database after it is processed
async fn delete_schedule(schedule_id: i32, db_url: &str) -> Result<(), Error> {
    match tokio::task::spawn_blocking({
        let db_url = db_url.to_string();
        move || {
            let mut connection = establish_connection(&db_url);
            diesel::delete(
                crate::schema::schedule::table.filter(crate::schema::schedule::id.eq(schedule_id)),
            )
            .execute(&mut connection)
        }
    })
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::from(e)),
    }
}
