use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::http::{header::AUTHORIZATION, HeaderMap};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::auth::models::{AuthResponse, AuthUser, LoginInput, RegisterInput},
    shared::{
        crypto::hash_password,
        errors::{bad_request, forbidden, internal_error, unauthorized, ApiError},
        validation::{normalize_email, normalize_username},
    },
};

use super::repo;

const ACCESS_TOKEN_TTL_SECONDS: usize = 15 * 60;

#[derive(Serialize, Deserialize)]
struct AccessClaims {
    sub: String,
    exp: usize,
    token_type: String,
}

pub async fn register_user(state: &AppState, payload: RegisterInput) -> Result<AuthResponse, ApiError> {
    validate_register_input(&payload)?;

    let normalized_email = normalize_email(&payload.email);
    if repo::email_exists(&state.pool, &normalized_email).await? {
        return Err(bad_request("Email is already registered"));
    }

    let normalized_username = normalize_username(&payload.username)?;
    if repo::username_exists(&state.pool, &normalized_username).await? {
        return Err(bad_request("Username is already taken"));
    }

    let password_hash = hash_password(&payload.password)?;
    let user_id = Uuid::new_v4();

    repo::insert_user(
        &state.pool,
        user_id,
        &normalized_email,
        &password_hash,
        &normalized_username,
        payload.display_name.trim(),
    )
    .await?;
    repo::insert_local_identity(&state.pool, user_id, &normalized_email).await?;
    repo::insert_default_role(&state.pool, user_id).await?;

    let user = get_auth_user_by_id(state, user_id).await?;
    issue_auth_response(state, user, None).await
}

pub async fn login_user(state: &AppState, payload: LoginInput) -> Result<AuthResponse, ApiError> {
    let normalized_email = normalize_email(&payload.email);

    if normalized_email.is_empty() || payload.password.is_empty() {
        return Err(bad_request("Email and password are required"));
    }

    let Some(login) =
        repo::find_user_login_by_email(&state.pool, &normalized_email).await?
    else {
        return Err(unauthorized("Invalid email or password"));
    };

    if !login.is_active {
        return Err(forbidden("User account is inactive"));
    }

    let Some(password_hash) = login.password_hash else {
        return Err(unauthorized("Invalid email or password"));
    };

    verify_password(&password_hash, &payload.password)?;

    let user = get_auth_user_by_id(state, login.id).await?;

    issue_auth_response(state, user, None).await
}

pub async fn refresh_auth(state: &AppState, refresh_token: &str) -> Result<AuthResponse, ApiError> {
    if refresh_token.trim().is_empty() {
        return Err(bad_request("Refresh token is required"));
    }

    let refresh_hash = hash_refresh_token(refresh_token);
    let Some(session) = repo::find_active_session_by_hash(&state.pool, &refresh_hash).await? else {
        return Err(unauthorized("Invalid or expired refresh token"));
    };

    let user = get_auth_user_by_id(state, session.user_id).await?;
    issue_auth_response(state, user, Some(session.id)).await
}

pub async fn logout_session(state: &AppState, refresh_token: &str) -> Result<(), ApiError> {
    if refresh_token.trim().is_empty() {
        return Err(bad_request("Refresh token is required"));
    }

    let refresh_hash = hash_refresh_token(refresh_token);
    repo::revoke_session_by_hash(&state.pool, &refresh_hash).await
}

pub fn require_authenticated_user_id(state: &AppState, headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| unauthorized("Missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| unauthorized("Authorization header must use Bearer token"))?;

    let token_data = decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| unauthorized("Invalid access token"))?;

    if token_data.claims.token_type != "access" {
        return Err(unauthorized("Invalid access token type"));
    }

    Uuid::parse_str(&token_data.claims.sub).map_err(|_| unauthorized("Invalid access token subject"))
}

pub fn maybe_authenticated_user_id(state: &AppState, headers: &HeaderMap) -> Option<Uuid> {
    require_authenticated_user_id(state, headers).ok()
}

pub async fn get_auth_user_by_id(state: &AppState, user_id: Uuid) -> Result<AuthUser, ApiError> {
    let Some(row) = repo::find_user_profile_by_id(&state.pool, user_id).await? else {
        return Err(unauthorized("User not found"));
    };

    if !row.is_active {
        return Err(forbidden("User account is inactive"));
    }

    let has_battlenet_identity = repo::has_provider_identity(&state.pool, user_id, "battlenet").await?;

    Ok(AuthUser {
        id: row.id,
        email: row.email,
        username: row.username,
        display_name: row.display_name,
        role: row.role,
        battletag: row.battletag,
        rank_tank: row.rank_tank,
        rank_dps: row.rank_dps,
        rank_support: row.rank_support,
        can_edit_battletag: !has_battlenet_identity,
    })
}

fn validate_register_input(payload: &RegisterInput) -> Result<(), ApiError> {
    let email = normalize_email(&payload.email);
    if email.is_empty() || !email.contains('@') {
        return Err(bad_request("A valid email is required"));
    }

    if payload.password.len() < 8 {
        return Err(bad_request("Password must be at least 8 characters long"));
    }

    if payload.password != payload.password_confirm {
        return Err(bad_request("Passwords do not match"));
    }

    if payload.display_name.trim().is_empty() {
        return Err(bad_request("Display name is required"));
    }

    normalize_username(&payload.username)?;

    Ok(())
}



fn verify_password(stored_hash: &str, password: &str) -> Result<(), ApiError> {
    let parsed_hash = PasswordHash::new(stored_hash).map_err(|_| unauthorized("Invalid email or password"))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| unauthorized("Invalid email or password"))
}

async fn issue_auth_response(
    state: &AppState,
    user: AuthUser,
    existing_session_id: Option<Uuid>,
) -> Result<AuthResponse, ApiError> {
    let access_token = build_access_token(user.id, &state.config.jwt_secret)?;
    let refresh_token = format!("{}.{}", Uuid::new_v4(), Uuid::new_v4());
    let refresh_hash = hash_refresh_token(&refresh_token);

    if let Some(session_id) = existing_session_id {
        repo::rotate_session(&state.pool, session_id, &refresh_hash).await?;
    } else {
        repo::create_session(&state.pool, user.id, &refresh_hash).await?;
    }

    Ok(AuthResponse {
        access_token,
        refresh_token,
        user,
    })
}

fn build_access_token(user_id: Uuid, jwt_secret: &str) -> Result<String, ApiError> {
    let now = current_unix_timestamp();
    let claims = AccessClaims {
        sub: user_id.to_string(),
        exp: now + ACCESS_TOKEN_TTL_SECONDS,
        token_type: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| internal_error(e))
}

fn hash_refresh_token(refresh_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    hex::encode(hasher.finalize())
}

fn current_unix_timestamp() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as usize)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Battle.net OAuth
// ---------------------------------------------------------------------------

const OAUTH_STATE_TTL_SECONDS: u64 = 600; // 10 minutes

/// Outcome of processing a Battle.net OAuth redirect.
pub enum BnetCallbackResult {
    /// New login or existing login — returns a fresh auth session.
    LoggedIn(AuthResponse),
    /// Linked to an existing account — returns the user's ID for redirect.
    Connected(Uuid),
}

// ── State building ──────────────────────────────────────────────────────────

/// Builds a CSRF state token for the login flow.
/// Format: `{ts_hex}.{nonce}.login.{sig}`
pub fn build_oauth_state(jwt_secret: &str) -> String {
    build_state_with_mode(jwt_secret, "login")
}

/// Builds a CSRF state token for the connect-to-existing-account flow.
/// Format: `{ts_hex}.{nonce}.connect:{user_id}.{sig}`
pub fn build_oauth_connect_state(jwt_secret: &str, user_id: Uuid) -> String {
    let mode_tag = format!("connect:{}", user_id.as_simple());
    build_state_with_mode(jwt_secret, &mode_tag)
}

fn build_state_with_mode(jwt_secret: &str, mode_tag: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let ts_hex = format!("{:x}", now);
    let nonce = Uuid::new_v4().simple().to_string();
    let sig = sha256_hex(&format!("{}.{}.{}.{}", ts_hex, nonce, mode_tag, jwt_secret));
    format!("{}.{}.{}.{}", ts_hex, nonce, mode_tag, sig)
}

enum OAuthMode {
    Login,
    Connect(Uuid),
}

/// Verifies the CSRF state and returns the embedded mode, or `None` if invalid/expired.
/// State format: `{ts_hex}.{nonce}.{mode_tag}.{sig}` where sig is after the last dot.
fn verify_and_parse_oauth_state(jwt_secret: &str, state: &str) -> Option<OAuthMode> {
    let last_dot = state.rfind('.')?;
    let (prefix, provided_sig) = (&state[..last_dot], &state[last_dot + 1..]);

    // prefix splits into exactly 3 parts: ts, nonce, mode_tag
    let parts: Vec<&str> = prefix.splitn(3, '.').collect();
    if parts.len() != 3 {
        return None;
    }
    let (ts_hex, nonce, mode_tag) = (parts[0], parts[1], parts[2]);

    let ts_val = u64::from_str_radix(ts_hex, 16).ok()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if now.saturating_sub(ts_val) > OAUTH_STATE_TTL_SECONDS {
        return None;
    }

    let expected_sig = sha256_hex(&format!("{}.{}.{}.{}", ts_hex, nonce, mode_tag, jwt_secret));
    if !constant_time_eq(provided_sig.as_bytes(), expected_sig.as_bytes()) {
        return None;
    }

    if mode_tag == "login" {
        Some(OAuthMode::Login)
    } else if let Some(uuid_str) = mode_tag.strip_prefix("connect:") {
        Uuid::parse_str(uuid_str).ok().map(OAuthMode::Connect)
    } else {
        None
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

// ── BNet API types ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct BnetTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct BnetUserInfo {
    sub: String,
    battletag: Option<String>,
    email: Option<String>,
}

// ── Shared helpers ──────────────────────────────────────────────────────────

async fn exchange_bnet_code(state: &AppState, code: &str) -> Result<(String, String, Option<String>), ApiError> {
    let client = reqwest::Client::new();

    let token_resp = client
        .post("https://oauth.battle.net/token")
        .basic_auth(
            &state.config.battlenet_client_id,
            Some(&state.config.battlenet_client_secret),
        )
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", state.config.battlenet_redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| internal_error(format!("BNet token exchange failed: {e}")))?;

    if !token_resp.status().is_success() {
        return Err(bad_request("Battle.net authentication failed"));
    }

    let token_data: BnetTokenResponse = token_resp
        .json()
        .await
        .map_err(|e| internal_error(format!("Failed to parse BNet token response: {e}")))?;

    let userinfo_resp = client
        .get("https://oauth.battle.net/userinfo")
        .bearer_auth(&token_data.access_token)
        .send()
        .await
        .map_err(|e| internal_error(format!("BNet userinfo request failed: {e}")))?;

    if !userinfo_resp.status().is_success() {
        return Err(internal_error("Failed to fetch Battle.net user info"));
    }

    let userinfo: BnetUserInfo = userinfo_resp
        .json()
        .await
        .map_err(|e| internal_error(format!("Failed to parse BNet userinfo: {e}")))?;

    let battletag = userinfo
        .battletag
        .unwrap_or_else(|| format!("Player#{}", &userinfo.sub[..6.min(userinfo.sub.len())]));

    Ok((userinfo.sub, battletag, userinfo.email))
}

// ── Public: OAuth redirect handler ─────────────────────────────────────────

/// Verifies the CSRF state, exchanges the code, and routes to login or connect logic.
pub async fn handle_battlenet_redirect(
    state: &AppState,
    code: &str,
    csrf_state: &str,
) -> Result<BnetCallbackResult, ApiError> {
    let mode = verify_and_parse_oauth_state(&state.config.jwt_secret, csrf_state)
        .ok_or_else(|| bad_request("Invalid or expired OAuth state"))?;

    let (sub, battletag, bnet_email) = exchange_bnet_code(state, code).await?;

    match mode {
        OAuthMode::Login => {
            let auth = handle_bnet_login(state, &sub, &battletag, bnet_email.as_deref()).await?;
            Ok(BnetCallbackResult::LoggedIn(auth))
        }
        OAuthMode::Connect(user_id) => {
            handle_bnet_connect(state, user_id, &sub, &battletag).await?;
            Ok(BnetCallbackResult::Connected(user_id))
        }
    }
}

async fn handle_bnet_login(
    state: &AppState,
    sub: &str,
    battletag: &str,
    bnet_email: Option<&str>,
) -> Result<AuthResponse, ApiError> {
    let user_id = match repo::find_user_id_by_bnet_sub(&state.pool, sub).await? {
        Some(id) => {
            repo::upsert_bnet_game_profile(&state.pool, id, sub, battletag).await?;
            id
        }
        None => {
            let email = bnet_email
                .filter(|e| !e.is_empty())
                .ok_or_else(|| bad_request("Battle.net did not provide an email address"))?
                .to_string();

            // If that email is already registered (e.g. a previously-disconnected BNet
            // account), re-link rather than trying to create a duplicate.
            if let Some(existing) = repo::find_user_login_by_email(&state.pool, &email).await? {
                repo::insert_bnet_identity(&state.pool, existing.id, sub).await?;
                repo::upsert_bnet_game_profile(&state.pool, existing.id, sub, battletag).await?;
                existing.id
            } else {
                let new_id = Uuid::new_v4();
                let (base_username, display_name) = username_from_battletag(battletag);
                let username = resolve_unique_username(&state.pool, &base_username).await?;
                repo::insert_bnet_user(&state.pool, new_id, &email, &username, &display_name).await?;
                repo::insert_bnet_identity(&state.pool, new_id, sub).await?;
                repo::insert_default_role(&state.pool, new_id).await?;
                repo::upsert_bnet_game_profile(&state.pool, new_id, sub, battletag).await?;
                new_id
            }
        }
    };

    let user = get_auth_user_by_id(state, user_id).await?;
    issue_auth_response(state, user, None).await
}

async fn handle_bnet_connect(
    state: &AppState,
    user_id: Uuid,
    sub: &str,
    battletag: &str,
) -> Result<(), ApiError> {
    if let Some(existing_id) = repo::find_user_id_by_bnet_sub(&state.pool, sub).await? {
        if existing_id != user_id {
            return Err(bad_request(
                "This Battle.net account is already linked to another profile",
            ));
        }
        // Re-linking to the same user: just refresh the game profile.
        repo::upsert_bnet_game_profile(&state.pool, user_id, sub, battletag).await?;
        return Ok(());
    }

    repo::insert_bnet_identity(&state.pool, user_id, sub).await?;
    repo::upsert_bnet_game_profile(&state.pool, user_id, sub, battletag).await?;
    Ok(())
}

// ── Public: connect-init URL builder ───────────────────────────────────────

/// Returns the Battle.net authorize URL for linking an existing account.
pub async fn battlenet_connect_init_url(
    state: &AppState,
    user_id: Uuid,
) -> Result<String, ApiError> {
    if state.config.battlenet_client_id.is_empty() {
        return Err(bad_request("Battle.net login is not configured"));
    }
    if repo::has_provider_identity(&state.pool, user_id, "battlenet").await? {
        return Err(bad_request(
            "A Battle.net account is already connected to this profile",
        ));
    }
    let csrf_state = build_oauth_connect_state(&state.config.jwt_secret, user_id);
    Ok(format!(
        "https://oauth.battle.net/authorize?client_id={}&scope=openid%20email&state={}&redirect_uri={}&response_type=code",
        urlencoding::encode(&state.config.battlenet_client_id),
        urlencoding::encode(&csrf_state),
        urlencoding::encode(&state.config.battlenet_redirect_uri),
    ))
}

// ── Public: disconnect ──────────────────────────────────────────────────────

pub async fn disconnect_battlenet(state: &AppState, user_id: Uuid) -> Result<(), ApiError> {
    if !repo::has_provider_identity(&state.pool, user_id, "battlenet").await? {
        return Err(bad_request("No Battle.net account is connected to this profile"));
    }
    repo::remove_bnet_identity(&state.pool, user_id).await?;
    repo::unlock_bnet_game_profile(&state.pool, user_id).await?;
    Ok(())
}

// ── Private: username helpers ───────────────────────────────────────────────

/// Derives a slug username and display name from a battletag like `CoolPlayer#1234`.
fn username_from_battletag(battletag: &str) -> (String, String) {
    let name_part = battletag.split('#').next().unwrap_or("player");
    let discriminator = battletag.split('#').nth(1).unwrap_or("");

    let slug: String = name_part
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() {
                Some(c.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect();

    let slug = if slug.is_empty() { "player".to_string() } else { slug };
    // Keep room for discriminator suffix (max total: 24 chars).
    let slug = slug[..slug.len().min(20)].to_string();
    let candidate = if discriminator.is_empty() {
        slug.clone()
    } else {
        let full = format!("{}{}", slug, discriminator);
        full[..full.len().min(24)].to_string()
    };

    (candidate, name_part.to_string())
}

/// Returns `base` if available, otherwise appends a numeric suffix until unique.
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

    // Extremely unlikely fallback.
    let suffix = &Uuid::new_v4().simple().to_string()[..8];
    Ok(format!("{}-{}", &base[..base.len().min(15)], suffix))
}
