use anyhow::{bail, Result};
use reqwest::Client;
use serde_json::{json, Value};
use tracing::debug;

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
        name: &str,
        description: &str,
        event_type: &str,
        start_date: Option<&str>,
        event_url: &str,
    ) -> Result<String> {
        let type_label = match event_type {
            "PUG" => "Pick-Up Game",
            "TOURNEY" => "Tournament",
            other => other,
        };

        let mut fields = vec![json!({"name": "Type", "value": type_label, "inline": true})];

        if let Some(date) = start_date {
            fields.push(json!({"name": "Starts", "value": date, "inline": true}));
        }

        let embed = json!({
            "title": name,
            "description": if description.is_empty() { Value::Null } else { Value::String(description.to_string()) },
            "url": event_url,
            "color": 0x5865F2,
            "fields": fields,
            "footer": {"text": "tornare.gg"},
        });

        let body = json!({"embeds": [embed]});

        debug!("Posting embed for {event_url}");

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
}
