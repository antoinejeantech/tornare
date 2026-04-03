mod discord;

use std::env;

use sqlx::postgres::PgListener;
use sqlx::{PgPool, Row};
use tracing::{error, info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bot_token = env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN must be set");
    let channel_id = env::var("DISCORD_CHANNEL_ID").expect("DISCORD_CHANNEL_ID must be set");
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "https://tornare.gg".to_string());

    info!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    info!("Database connected");

    let http = discord::DiscordHttp::new(bot_token);

    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("event_published").await?;
    info!("Listening for event_published notifications");

    loop {
        match listener.recv().await {
            Ok(notification) => {
                let payload = notification.payload();
                match payload.parse::<Uuid>() {
                    Ok(event_id) => {
                        if let Err(e) = handle_event_published(
                            &pool,
                            &http,
                            &channel_id,
                            &frontend_url,
                            event_id,
                        )
                        .await
                        {
                            error!("Failed to handle event_published {event_id}: {e:#}");
                        }
                    }
                    Err(_) => warn!("Received invalid UUID payload: {payload}"),
                }
            }
            Err(e) => {
                // PgListener reconnects automatically on the next recv() call.
                error!("Listener error: {e:#}");
            }
        }
    }
}

async fn handle_event_published(
    pool: &PgPool,
    http: &discord::DiscordHttp,
    channel_id: &str,
    frontend_url: &str,
    event_id: Uuid,
) -> anyhow::Result<()> {
    let row = sqlx::query(
        "SELECT name, description, event_type, start_date, discord_message_id \
         FROM events WHERE id = $1",
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        warn!("Event {event_id} not found, skipping");
        return Ok(());
    };

    // If we already posted, skip to avoid duplicates on bot restart.
    let already_posted: Option<String> = row.try_get("discord_message_id").ok().flatten();
    if already_posted.is_some() {
        info!("Event {event_id} already has a Discord message, skipping");
        return Ok(());
    }

    let name: String = row.try_get("name")?;
    let description: String = row.try_get("description")?;
    let event_type: String = row.try_get("event_type")?;
    let start_date: Option<String> = row.try_get("start_date").ok().flatten();

    let event_url = format!("{frontend_url}/events/{event_id}");

    let message_id = http
        .post_event_embed(
            channel_id,
            &name,
            &description,
            &event_type,
            start_date.as_deref(),
            &event_url,
        )
        .await?;

    sqlx::query("UPDATE events SET discord_message_id = $1 WHERE id = $2")
        .bind(&message_id)
        .bind(event_id)
        .execute(pool)
        .await?;

    info!("Posted Discord message {message_id} for event '{name}' ({event_id})");
    Ok(())
}
