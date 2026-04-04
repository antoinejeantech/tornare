use reqwest::Client;
use serde_json::json;
use tracing::{info, warn};

/// Register global slash commands with Discord via bulk PUT.
pub async fn register(bot_token: &str, application_id: &str) -> anyhow::Result<()> {
    let url = format!("https://discord.com/api/v10/applications/{application_id}/commands");

    let payload = json!([
        {
            "name": "setup",
            "description": "Connect this channel for Tornare event announcements",
            "options": [{
                "type": 7,
                "name": "channel",
                "description": "Channel to post announcements in (defaults to current channel)",
                "required": false,
                "channel_types": [0]
            }],
            "default_member_permissions": "32"
        },
        {
            "name": "unsetup",
            "description": "Disconnect this server from Tornare announcements",
            "default_member_permissions": "32"
        },
        {
            "name": "help",
            "description": "Show Tornare bot commands and info"
        }
    ]);

    let resp = Client::new()
        .put(&url)
        .header("Authorization", format!("Bot {bot_token}"))
        .json(&payload)
        .send()
        .await?;

    if resp.status().is_success() {
        info!("Slash commands registered successfully");
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        warn!("Discord returned {status} when registering commands: {body}");
    }
    Ok(())
}
