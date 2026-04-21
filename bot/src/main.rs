use tornare_bot::announcements;
use tornare_bot::commands;
use tornare_bot::discord;

use std::env;
use std::time::Duration;

use sqlx::PgPool;
use tokio::time;
use tracing::{info, warn};

const POLL_INTERVAL_SECS_DEFAULT: u64 = 60;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bot_token = env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN must be set");
    let poll_interval_secs = env::var("POLL_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(POLL_INTERVAL_SECS_DEFAULT);
    let application_id = env::var("DISCORD_BOT_APPLICATION_ID").ok();
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    info!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    info!("Database connected. Polling every {poll_interval_secs}s for unpublished events.");

    let http = discord::DiscordHttp::new(bot_token.clone());

    // Register slash commands only when REGISTER_COMMANDS=true.
    // Commands are global and persist on Discord's side, so this only needs
    // to run once after changes — not on every startup.
    let register_commands = env::var("REGISTER_COMMANDS")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);

    if register_commands {
        if let Some(ref app_id) = application_id {
            info!("Registering slash commands...");
            if let Err(e) = commands::register(&bot_token, app_id).await {
                warn!("Failed to register slash commands: {e:#}");
            } else {
                info!("Slash commands registered.");
            }
        } else {
            warn!("REGISTER_COMMANDS=true but DISCORD_BOT_APPLICATION_ID is not set — skipping");
        }
    } else {
        info!("Skipping slash command registration (set REGISTER_COMMANDS=true to register)");
    }

    let mut interval = time::interval(Duration::from_secs(poll_interval_secs));
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        announcements::run_poll(&pool, &http, &frontend_url).await;
    }
}
