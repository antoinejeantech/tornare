use anyhow::{bail, Result};
use reqwest::Client;
use serde_json::{json, Value};
use tracing::debug;

pub struct EventEmbed {
    pub name: String,
    pub description: String,
    pub event_type: String,
    pub format: String,
    pub max_players: i32,
    pub start_date: Option<String>,
    pub organizer: Option<String>,
    pub event_url: String,
    /// Set when the event uses private (token-based) signup.
    pub join_url: Option<String>,
}

pub struct DiscordHttp {
    client: Client,
    token: String,
}

impl DiscordHttp {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    pub async fn post_event_embed(
        &self,
        channel_id: &str,
        ev: &EventEmbed,
    ) -> Result<String> {
        const TORNARE_LOGO: &str = "https://tornare.vercel.app/pwa-icon-192.png";

        let (type_label, author_name, color) = match ev.event_type.as_str() {
            "PUG"     => ("Pick-Up Game", "\u{1F3AE}  New Pick-Up Game", 0x5865F2_u32),
            "TOURNEY" => ("Tournament",   "\u{1F3C6}  New Tournament",   0xF0A500_u32),
            other     => (other,          "\u{1F4C5}  New Event",        0x5865F2_u32),
        };

        // Row 1: type · format · capacity  (3 inline = one Discord row)
        let mut fields = vec![
            json!({"name": "Type",     "value": type_label,                             "inline": true}),
            json!({"name": "Format",   "value": ev.format.as_str(),                     "inline": true}),
            json!({"name": "Capacity", "value": format!("{} players", ev.max_players),  "inline": true}),
        ];

        // Row 2 (optional): starts · organizer
        if let Some(date) = &ev.start_date {
            fields.push(json!({"name": "\u{1F4C5} Starts", "value": date.as_str(), "inline": true}));
        }
        if let Some(org) = &ev.organizer {
            fields.push(json!({"name": "Organizer", "value": org.as_str(), "inline": true}));
        }

        let description = if ev.description.is_empty() {
            Value::Null
        } else {
            Value::String(ev.description.clone())
        };

        let timestamp = chrono::Utc::now().to_rfc3339();

        let embed = json!({
            "author":      {"name": author_name},
            "title":       ev.name.as_str(),
            "url":         ev.event_url.as_str(),
            "description": description,
            "color":       color,
            "fields":      fields,
            "thumbnail":   {"url": TORNARE_LOGO},
            "footer":      {"text": "Tornare", "icon_url": TORNARE_LOGO},
            "timestamp":   timestamp,
        });

        // Build action row with link buttons.
        // style 5 = LINK — opens URL, requires no interaction handler.
        let mut buttons = vec![
            json!({
                "type":  2,
                "style": 5,
                "label": "View Event",
                "url":   ev.event_url.as_str(),
                "emoji": {"name": "\u{1F4C5}"},
            }),
        ];
        if let Some(url) = &ev.join_url {
            buttons.push(json!({
                "type":  2,
                "style": 5,
                "label": "Join Event",
                "url":   url.as_str(),
                "emoji": {"name": "\u{1F4DD}"},
            }));
        }
        let components = vec![json!({"type": 1, "components": buttons})];

        let body = json!({"embeds": [embed], "components": components});

        debug!("Posting embed for {}", ev.event_url);

        let resp = self
            .client
            .post(format!(
                "https://discord.com/api/v10/channels/{channel_id}/messages"
            ))
            .header("Authorization", format!("Bot {}", self.token))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Discord API error {status}: {text}");
        }

        let data: Value = resp.json().await?;
        let message_id = data["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No id field in Discord response"))?
            .to_string();

        Ok(message_id)
    }

    /// Fetch the guild name from Discord for a given guild ID.
    pub async fn fetch_guild_name(&self, guild_id: &str) -> Result<Option<String>> {
        let resp = self
            .client
            .get(format!("https://discord.com/api/v10/guilds/{guild_id}"))
            .header("Authorization", format!("Bot {}", self.token))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let data: Value = resp.json().await?;
        Ok(data["name"].as_str().map(|s| s.to_string()))
    }

    /// Check whether the bot is still a member of the given guild.
    /// Returns `false` on 403/404 (bot removed/banned or guild gone).
    /// Returns `true` on 200. Network errors propagate as `Err`.
    pub async fn is_bot_in_guild(&self, guild_id: &str) -> Result<bool> {
        let resp = self
            .client
            .get(format!("https://discord.com/api/v10/guilds/{guild_id}"))
            .header("Authorization", format!("Bot {}", self.token))
            .send()
            .await?;

        Ok(resp.status().is_success())
    }
}
