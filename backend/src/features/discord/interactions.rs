use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use ed25519_dalek::{Signature, VerifyingKey};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info, warn};

use crate::app::state::AppState;

use super::repo;

// ---------------------------------------------------------------------------
// Discord interaction types (subset we care about)
// ---------------------------------------------------------------------------

const INTERACTION_PING: u64 = 1;
const INTERACTION_APPLICATION_COMMAND: u64 = 2;
const RESPONSE_PONG: u64 = 1;
const RESPONSE_CHANNEL_MESSAGE: u64 = 4;
const RESPONSE_EPHEMERAL: u64 = 64; // message_flags

#[derive(Deserialize)]
struct Interaction {
    #[serde(rename = "type")]
    kind: u64,
    data: Option<CommandData>,
    guild_id: Option<String>,
    channel_id: Option<String>,
    member: Option<Member>,
}

#[derive(Deserialize)]
struct CommandData {
    name: String,
    options: Option<Vec<CommandOption>>,
    #[allow(dead_code)]
    resolved: Option<Value>,
}

#[derive(Deserialize)]
struct CommandOption {
    name: String,
    value: Option<Value>,
}

#[derive(Deserialize)]
struct Member {
    user: Option<DiscordUser>,
    permissions: Option<String>,
}

impl Member {
    fn is_admin(&self) -> bool {
        self.permissions
            .as_deref()
            .and_then(|p| p.parse::<u64>().ok())
            .map(|bits| bits & 0x8 != 0)
            .unwrap_or(false)
    }

    fn discord_user_id(&self) -> &str {
        self.user.as_ref().map(|u| u.id.as_str()).unwrap_or("")
    }
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String,
}

#[allow(dead_code)]
#[derive(Serialize)]
struct InteractionResponse {
    #[serde(rename = "type")]
    kind: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<InteractionCallbackData>,
}

#[derive(Serialize)]
struct InteractionCallbackData {
    content: String,
    flags: u64,
}

// ---------------------------------------------------------------------------
// Signature verification
// ---------------------------------------------------------------------------

fn verify_discord_signature(
    public_key_hex: &str,
    signature_hex: &str,
    timestamp: &str,
    body: &[u8],
) -> bool {
    let Ok(pub_key_bytes) = hex::decode(public_key_hex) else {
        return false;
    };
    let Ok(sig_bytes) = hex::decode(signature_hex) else {
        return false;
    };
    let Ok(key_array): Result<[u8; 32], _> = pub_key_bytes.try_into() else {
        return false;
    };
    let Ok(sig_array): Result<[u8; 64], _> = sig_bytes.try_into() else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&key_array) else {
        return false;
    };
    let signature = Signature::from_bytes(&sig_array);

    let mut message = timestamp.as_bytes().to_vec();
    message.extend_from_slice(body);

    use ed25519_dalek::Verifier;
    verifying_key.verify(&message, &signature).is_ok()
}

/// Reject interactions whose timestamp is more than 5 minutes old to prevent replay attacks.
fn is_timestamp_fresh(timestamp: &str) -> bool {
    timestamp
        .parse::<i64>()
        .map(|ts| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            (now - ts).abs() < 300
        })
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn handle_interactions(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    // --- Verify signature ---
    let signature = headers
        .get("x-signature-ed25519")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let timestamp = headers
        .get("x-signature-timestamp")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if state.config.discord_bot_public_key.is_empty() {
        warn!("DISCORD_BOT_PUBLIC_KEY is not configured — rejecting interaction");
        return Err(StatusCode::UNAUTHORIZED);
    }
    if !is_timestamp_fresh(timestamp) {
        warn!("Discord interaction timestamp is stale — possible replay attack");
        return Err(StatusCode::UNAUTHORIZED);
    }
    if !verify_discord_signature(
        &state.config.discord_bot_public_key,
        signature,
        timestamp,
        &body,
    ) {
        warn!("Invalid Discord interaction signature");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let interaction: Interaction = match serde_json::from_slice(&body) {
        Ok(i) => i,
        Err(e) => {
            error!("Failed to parse Discord interaction: {e}");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // --- Ping (Discord liveness check) ---
    if interaction.kind == INTERACTION_PING {
        return Ok(Json(serde_json::json!({ "type": RESPONSE_PONG })));
    }

    // --- Slash commands ---
    if interaction.kind == INTERACTION_APPLICATION_COMMAND {
        if let Some(data) = &interaction.data {
            match data.name.as_str() {
                "setup"   => return handle_setup(&state, &interaction, data).await,
                "unsetup" => return handle_unsetup(&state, &interaction).await,
                "help"    => return Ok(handle_help(&state.config.frontend_url)),
                _ => {}
            }
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

async fn handle_setup(
    state: &AppState,
    interaction: &Interaction,
    data: &CommandData,
) -> Result<Json<Value>, StatusCode> {
    let guild_id = match require_guild_id(interaction) {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    // The channel option: if provided use it, otherwise use the current channel.
    let channel_id = data
        .options
        .as_ref()
        .and_then(|opts| opts.iter().find(|o| o.name == "channel"))
        .and_then(|o| o.value.as_ref())
        .and_then(|v: &Value| v.as_str())
        .map(String::from)
        .or_else(|| interaction.channel_id.clone())
        .unwrap_or_default();

    if channel_id.is_empty() {
        return Ok(ephemeral("⚠️ Could not determine the target channel."));
    }

    let member = interaction.member.as_ref();

    // Only server admins can configure the bot.
    if !member.map(|m| m.is_admin()).unwrap_or(false) {
        return Ok(ephemeral(
            "⚠️ Only server administrators can configure Tornare.",
        ));
    }

    let discord_user_id = member.map(|m| m.discord_user_id()).unwrap_or("");

    // Require a linked Tornare account — no anonymous guild registration.
    let has_account: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM auth_identities WHERE provider = 'discord' AND provider_user_id = $1)",
    )
    .bind(discord_user_id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);

    if !has_account {
        return Ok(ephemeral(&format!(
            "⚠️ You need a Tornare account with Discord connected to use this command. \
             Visit {} to create one and link your Discord.",
            state.config.frontend_url
        )));
    }

    let guild_name: Option<&str> = None; // bot will backfill via fetch_guild_name

    match repo::upsert_guild_from_slash(
        &state.pool,
        &guild_id,
        guild_name,
        &channel_id,
        discord_user_id,
    )
    .await
    {
        Ok(_) => {
            info!("Guild {guild_id} configured via /setup → channel {channel_id}");
            match check_channel_permissions(&state.config.discord_bot_token, &channel_id).await {
                ChannelAccess::Ok => Ok(ephemeral(&format!(
                    "✅ Tornare will post new event announcements in <#{channel_id}>."
                ))),
                ChannelAccess::MissingPermissions => Ok(ephemeral(&format!(
                    "✅ Server configured, but the bot is missing permissions in <#{channel_id}>.\n\
                     Please make sure the bot has **View Channel**, **Send Messages**, and **Embed Links** in that channel."
                ))),
                ChannelAccess::Error => Ok(ephemeral(&format!(
                    "✅ Tornare will post new event announcements in <#{channel_id}>.\n\
                     ⚠️ Could not verify channel permissions — make sure the bot has access."
                ))),
            }
        }
        Err(_) => Ok(ephemeral(
            "❌ Failed to save the configuration. Please try again.",
        )),
    }
}

enum ChannelAccess {
    Ok,
    MissingPermissions,
    Error,
}

async fn check_channel_permissions(bot_token: &str, channel_id: &str) -> ChannelAccess {
    if bot_token.is_empty() {
        return ChannelAccess::Error;
    }
    let client = Client::new();
    match client
        .get(format!("https://discord.com/api/v10/channels/{channel_id}"))
        .header("Authorization", format!("Bot {bot_token}"))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => ChannelAccess::Ok,
        Ok(resp) if resp.status() == 403 => ChannelAccess::MissingPermissions,
        Ok(resp) => {
            warn!("Unexpected status checking channel {channel_id}: {}", resp.status());
            ChannelAccess::Error
        }
        Err(e) => {
            error!("Failed to check channel permissions for {channel_id}: {e:#}");
            ChannelAccess::Error
        }
    }
}

async fn handle_unsetup(
    state: &AppState,
    interaction: &Interaction,
) -> Result<Json<Value>, StatusCode> {
    let guild_id = match require_guild_id(interaction) {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    let member = interaction.member.as_ref();
    let discord_user_id = member.map(|m| m.discord_user_id()).unwrap_or("");

    // Only server admins can disconnect the bot.
    if !member.map(|m| m.is_admin()).unwrap_or(false) {
        return Ok(ephemeral(
            "⚠️ Only server administrators can disconnect Tornare.",
        ));
    }

    // Only the owner (the user whose Discord identity matches) may unsetup.
    let owner_user_id: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM auth_identities \
         WHERE provider = 'discord' AND provider_user_id = $1 LIMIT 1",
    )
    .bind(discord_user_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    let Some(uid) = owner_user_id else {
        return Ok(ephemeral(&format!(
            "⚠️ Your Discord account is not linked to a Tornare account. \
             Visit {} to connect it first.",
            state.config.frontend_url
        )));
    };

    match repo::delete_guild_by_owner(&state.pool, uid, &guild_id).await {
        Ok(true) => {
            info!("Guild {guild_id} disconnected via /unsetup by user {uid}");
            Ok(ephemeral("✅ This server has been disconnected from Tornare."))
        }
        Ok(false) => Ok(ephemeral(
            "⚠️ No configuration found for this server. Nothing to remove.",
        )),
        Err(_) => Ok(ephemeral("❌ Failed to remove configuration. Please try again.")),
    }
}

fn require_guild_id(interaction: &Interaction) -> Result<String, Json<Value>> {
    interaction
        .guild_id
        .clone()
        .ok_or_else(|| ephemeral("⚠️ This command must be used inside a server."))
}

fn ephemeral(content: &str) -> Json<Value> {    Json(serde_json::json!({
        "type": RESPONSE_CHANNEL_MESSAGE,
        "data": {
            "content": content,
            "flags": RESPONSE_EPHEMERAL,
        }
    }))
}

fn handle_help(frontend_url: &str) -> Json<Value> {
    let content = format!("\
## 🏆 Tornare Bot
Automatically posts event announcements to your Discord server.

**Commands**
`/setup [channel]` — Connect a channel for event announcements. Requires **Administrator** permission.
`/unsetup` — Disconnect this server from Tornare.
`/help` — Show this message.

**Managing announcements**
Visit your server's Discord Bot settings page on [{frontend_url}]({frontend_url}) to toggle announcements on/off or disconnect the bot.");

    Json(serde_json::json!({
        "type": RESPONSE_CHANNEL_MESSAGE,
        "data": {
            "content": content,
            "flags": RESPONSE_EPHEMERAL,
        }
    }))
}
