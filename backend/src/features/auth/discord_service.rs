use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::auth::models::AuthResponse,
    shared::{errors::{ApiError, bad_request, internal_error}, validation::normalize_email},
};

use super::repo;
use super::service;

const OAUTH_STATE_TTL_SECONDS: usize = 600;
const DISCORD_HTTP_CONNECT_TIMEOUT_SECONDS: u64 = 5;
const DISCORD_HTTP_REQUEST_TIMEOUT_SECONDS: u64 = 15;

pub enum DiscordCallbackResult {
    LoggedIn(AuthResponse),
    Connected(Uuid),
}

#[derive(Serialize, Deserialize)]
struct DiscordOAuthStateClaims {
    exp: usize,
    token_type: String,
    mode: String,
    user_id: Option<String>,
    nonce: String,
}

#[derive(Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct DiscordUserInfo {
    id: String,
    username: String,
    global_name: Option<String>,
    email: Option<String>,
}

enum OAuthMode {
    Login,
    Connect(Uuid),
}

pub fn build_discord_oauth_state(jwt_secret: &str) -> Result<(String, String), ApiError> {
    build_state_with_mode(jwt_secret, "login", None)
}

pub fn build_discord_connect_state(
    jwt_secret: &str,
    user_id: Uuid,
) -> Result<(String, String), ApiError> {
    build_state_with_mode(jwt_secret, "connect", Some(user_id))
}

pub async fn handle_discord_redirect(
    state: &AppState,
    code: &str,
    csrf_state: &str,
    nonce: &str,
) -> Result<DiscordCallbackResult, ApiError> {
    info!("handling discord oauth redirect");
    let mode = verify_and_parse_oauth_state(&state.config.jwt_secret, csrf_state, nonce)
        .ok_or_else(|| {
            warn!("discord oauth state invalid or expired");
            bad_request("Invalid or expired OAuth state")
        })?;

    let (sub, discord_username, global_name, email) =
        exchange_discord_code(state, code).await?;

    match mode {
        OAuthMode::Login => {
            info!("discord oauth mode=login");
            if let Some(existing_user_id) =
                repo::find_user_id_by_discord_sub(&state.pool, &sub).await?
            {
                info!(
                    %existing_user_id,
                    "discord login matched existing sub mapping; issuing auth"
                );
                repo::ensure_discord_identity(&state.pool, existing_user_id, &sub, &discord_username).await?;
                let user = service::get_auth_user_by_id(state, existing_user_id).await?;
                let auth = service::issue_auth_response(state, user, None).await?;
                return Ok(DiscordCallbackResult::LoggedIn(auth));
            }

            let raw_email = email.ok_or_else(|| {
                warn!("discord login: no email returned by provider");
                bad_request(
                    "Discord did not provide an email address. Make sure your Discord account has a verified email.",
                )
            })?;
            let email = normalize_email(&raw_email)?;

            // If an account with this email exists → ask them to log in and connect from profile
            if repo::find_user_login_by_email(&state.pool, &email).await?.is_some() {
                return Err(bad_request(
                    "An account with this email already exists. Please log in with your email and password, then connect Discord from your profile settings.",
                ));
            }

            let user_id = create_discord_user(state, &sub, &discord_username, &global_name, &email).await?;
            let user = service::get_auth_user_by_id(state, user_id).await?;
            let auth = service::issue_auth_response(state, user, None).await?;
            Ok(DiscordCallbackResult::LoggedIn(auth))
        }
        OAuthMode::Connect(user_id) => {
            info!(%user_id, "discord oauth mode=connect");
            handle_discord_connect(state, user_id, &sub, &discord_username).await?;
            Ok(DiscordCallbackResult::Connected(user_id))
        }
    }
}

pub async fn discord_connect_init_url(
    state: &AppState,
    user_id: Uuid,
) -> Result<(String, String), ApiError> {
    let oauth_not_configured = state.config.discord_client_id.trim().is_empty()
        || state.config.discord_client_secret.trim().is_empty()
        || state.config.discord_redirect_uri.trim().is_empty();

    if oauth_not_configured {
        return Err(bad_request("Discord login is not configured"));
    }
    if repo::has_provider_identity(&state.pool, user_id, "discord").await? {
        return Err(bad_request(
            "A Discord account is already connected to this profile",
        ));
    }
    let (csrf_state, nonce) = build_discord_connect_state(&state.config.jwt_secret, user_id)?;
    let url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&scope=identify+email&state={}&redirect_uri={}&response_type=code",
        urlencoding::encode(&state.config.discord_client_id),
        urlencoding::encode(&csrf_state),
        urlencoding::encode(&state.config.discord_redirect_uri),
    );
    Ok((url, nonce))
}

pub async fn disconnect_discord(state: &AppState, user_id: Uuid) -> Result<(), ApiError> {
    if !repo::has_provider_identity(&state.pool, user_id, "discord").await? {
        return Err(bad_request("No Discord account is connected to this profile"));
    }
    if !repo::user_has_password(&state.pool, user_id).await? {
        return Err(bad_request(
            "Set a password for your account before disconnecting Discord, otherwise you will lose access",
        ));
    }
    repo::remove_discord_identity(&state.pool, user_id).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn build_state_with_mode(
    jwt_secret: &str,
    mode: &str,
    user_id: Option<Uuid>,
) -> Result<(String, String), ApiError> {
    let nonce = Uuid::new_v4().simple().to_string();
    let claims = DiscordOAuthStateClaims {
        exp: current_unix_timestamp() + OAUTH_STATE_TTL_SECONDS,
        token_type: "discord_oauth_state".to_string(),
        mode: mode.to_string(),
        user_id: user_id.map(|v| v.to_string()),
        nonce: nonce.clone(),
    };
    let jwt = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(internal_error)?;
    Ok((jwt, nonce))
}

fn verify_and_parse_oauth_state(
    jwt_secret: &str,
    state: &str,
    nonce: &str,
) -> Option<OAuthMode> {
    let claims = decode::<DiscordOAuthStateClaims>(
        state,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .ok()?
    .claims;

    if claims.token_type != "discord_oauth_state" {
        return None;
    }
    if claims.nonce != nonce {
        return None;
    }

    match claims.mode.as_str() {
        "login" => Some(OAuthMode::Login),
        "connect" => claims
            .user_id
            .as_deref()
            .and_then(|v| Uuid::parse_str(v).ok())
            .map(OAuthMode::Connect),
        _ => None,
    }
}

fn build_discord_http_client() -> Result<reqwest::Client, ApiError> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(DISCORD_HTTP_CONNECT_TIMEOUT_SECONDS))
        .timeout(Duration::from_secs(DISCORD_HTTP_REQUEST_TIMEOUT_SECONDS))
        .build()
        .map_err(internal_error)
}

async fn exchange_discord_code(
    state: &AppState,
    code: &str,
) -> Result<(String, String, Option<String>, Option<String>), ApiError> {
    info!("discord token exchange started");
    let client = build_discord_http_client()?;

    let token_resp = client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", state.config.discord_client_id.as_str()),
            ("client_secret", state.config.discord_client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", state.config.discord_redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "discord token exchange request failed");
            internal_error(format!("Discord token exchange failed: {e}"))
        })?;

    if !token_resp.status().is_success() {
        let status = token_resp.status();
        let body = token_resp.text().await.unwrap_or_default();
        warn!(%status, %body, "discord token exchange returned non-success status");
        return Err(bad_request("Discord authentication failed"));
    }

    let token_data: DiscordTokenResponse = token_resp.json().await.map_err(|e| {
        error!(error = %e, "failed to parse discord token response");
        internal_error(format!("Failed to parse Discord token response: {e}"))
    })?;

    info!("discord token exchange succeeded");

    let userinfo_resp = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(&token_data.access_token)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "discord userinfo request failed");
            internal_error(format!("Discord userinfo request failed: {e}"))
        })?;

    if !userinfo_resp.status().is_success() {
        let status = userinfo_resp.status();
        let body = userinfo_resp.text().await.unwrap_or_default();
        warn!(%status, %body, "discord userinfo returned non-success status");
        return Err(internal_error("Failed to fetch Discord user info"));
    }

    let userinfo: DiscordUserInfo = userinfo_resp.json().await.map_err(|e| {
        error!(error = %e, "failed to parse discord userinfo response");
        internal_error(format!("Failed to parse Discord userinfo: {e}"))
    })?;

    info!("discord userinfo resolved");

    Ok((
        userinfo.id,
        userinfo.username,
        userinfo.global_name,
        userinfo.email,
    ))
}

async fn create_discord_user(
    state: &AppState,
    sub: &str,
    discord_username: &str,
    global_name: &Option<String>,
    email: &str,
) -> Result<Uuid, ApiError> {
    let display_name = global_name
        .as_deref()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or(discord_username);

    let base_username = slugify_discord_username(discord_username);
    let username = resolve_unique_username(&state.pool, &base_username).await?;

    let user_id = Uuid::new_v4();
    repo::insert_discord_user(&state.pool, user_id, email, &username, display_name).await?;
    repo::ensure_discord_identity(&state.pool, user_id, sub, discord_username).await?;
    repo::insert_default_role(&state.pool, user_id).await?;
    info!(%user_id, "discord login created new user");
    Ok(user_id)
}

async fn handle_discord_connect(
    state: &AppState,
    user_id: Uuid,
    sub: &str,
    username: &str,
) -> Result<(), ApiError> {
    info!(%user_id, "handling discord connect");
    if let Some(existing_id) = repo::find_user_id_by_discord_sub(&state.pool, sub).await? {
        if existing_id != user_id {
            warn!(%user_id, %existing_id, "discord connect rejected: identity linked to another user");
            return Err(bad_request(
                "This Discord account is already linked to another profile",
            ));
        }
        repo::ensure_discord_identity(&state.pool, user_id, sub, username).await?;
        info!(%user_id, "discord connect re-linked existing identity");
        return Ok(());
    }
    repo::ensure_discord_identity(&state.pool, user_id, sub, username).await?;
    info!(%user_id, "discord connect linked new identity");
    Ok(())
}

fn slugify_discord_username(username: &str) -> String {
    let slug: String = username
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                Some(c.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect();
    let slug = if slug.is_empty() {
        "player".to_string()
    } else {
        slug
    };
    slug[..slug.len().min(24)].to_string()
}

async fn resolve_unique_username(pool: &PgPool, base: &str) -> Result<String, ApiError> {
    if !repo::username_exists(pool, base).await? {
        return Ok(base.to_string());
    }
    for i in 1..=99u32 {
        let candidate = format!("{}{}", &base[..base.len().min(22)], i);
        if !repo::username_exists(pool, &candidate).await? {
            return Ok(candidate);
        }
    }
    let suffix = &Uuid::new_v4().simple().to_string()[..8];
    Ok(format!("{}-{}", &base[..base.len().min(15)], suffix))
}

fn current_unix_timestamp() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0)
}
