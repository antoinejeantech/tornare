use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::auth::models::AuthResponse,
    shared::{
        errors::{bad_request, internal_error, ApiError},
        validation::normalize_email,
    },
};

use super::repo;
use super::service;

const OAUTH_STATE_TTL_SECONDS: usize = 600;
const BNET_PENDING_SIGNUP_TTL_SECONDS: usize = 10 * 60;
const BNET_HTTP_CONNECT_TIMEOUT_SECONDS: u64 = 5;
const BNET_HTTP_REQUEST_TIMEOUT_SECONDS: u64 = 15;

pub enum BnetCallbackResult {
    LoggedIn(AuthResponse),
    Connected(Uuid),
    RequiresEmail { pending_token: String, battletag: String },
}

#[derive(Serialize, Deserialize)]
struct BnetPendingSignupClaims {
    sub: String,
    battletag: String,
    exp: usize,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
struct BnetOAuthStateClaims {
    exp: usize,
    token_type: String,
    mode: String,
    user_id: Option<String>,
    nonce: String,
}

#[derive(Deserialize)]
struct BnetTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct BnetUserInfo {
    sub: String,
    battletag: Option<String>,
}

enum OAuthMode {
    Login,
    Connect(Uuid),
}

pub fn build_oauth_state(jwt_secret: &str) -> Result<(String, String), ApiError> {
    build_state_with_mode(jwt_secret, "login", None)
}

pub fn build_oauth_connect_state(jwt_secret: &str, user_id: Uuid) -> Result<(String, String), ApiError> {
    build_state_with_mode(jwt_secret, "connect", Some(user_id))
}

pub async fn handle_battlenet_redirect(
    state: &AppState,
    code: &str,
    csrf_state: &str,
    nonce: &str,
) -> Result<BnetCallbackResult, ApiError> {
    info!("handling battlenet oauth redirect");
    let mode = verify_and_parse_oauth_state(&state.config.jwt_secret, csrf_state, nonce)
        .ok_or_else(|| {
            warn!("battlenet oauth state invalid or expired");
            bad_request("Invalid or expired OAuth state")
        })?;

    let (sub, battletag) = exchange_bnet_code(state, code).await?;

    match mode {
        OAuthMode::Login => {
            info!("battlenet oauth mode=login");
            if let Some(existing_user_id) = repo::find_user_id_by_bnet_sub(&state.pool, &sub).await? {
                info!(
                    %existing_user_id,
                    "battlenet login matched existing sub mapping; issuing auth"
                );
                repo::ensure_bnet_identity(&state.pool, existing_user_id, &sub).await?;
                repo::upsert_bnet_game_profile(&state.pool, existing_user_id, &sub, &battletag).await?;
                let user = service::get_auth_user_by_id(state, existing_user_id).await?;
                let auth = service::issue_auth_response(state, user, None).await?;
                return Ok(BnetCallbackResult::LoggedIn(auth));
            }

            let pending_token = build_bnet_pending_signup_token(
                &state.config.jwt_secret,
                &sub,
                &battletag,
            )?;
            Ok(BnetCallbackResult::RequiresEmail {
                pending_token,
                battletag,
            })
        }
        OAuthMode::Connect(user_id) => {
            info!(%user_id, "battlenet oauth mode=connect");
            handle_bnet_connect(state, user_id, &sub, &battletag).await?;
            Ok(BnetCallbackResult::Connected(user_id))
        }
    }
}

pub async fn complete_battlenet_signup(
    state: &AppState,
    pending_token: &str,
    email: &str,
) -> Result<AuthResponse, ApiError> {
    if pending_token.trim().is_empty() {
        return Err(bad_request("Missing pending signup token"));
    }

    let email = normalize_email(email)?;

    let claims = decode::<BnetPendingSignupClaims>(
        pending_token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| bad_request("Invalid or expired pending signup token"))?
    .claims;

    if claims.token_type != "bnet_pending_signup" {
        return Err(bad_request("Invalid pending signup token type"));
    }

    info!("completing battlenet signup after email collection");

    let user_id = upsert_or_create_bnet_user(state, &claims.sub, &claims.battletag, &email).await?;

    let user = service::get_auth_user_by_id(state, user_id).await?;
    service::issue_auth_response(state, user, None).await
}

pub async fn battlenet_connect_init_url(
    state: &AppState,
    user_id: Uuid,
) -> Result<(String, String), ApiError> {
    let oauth_not_configured = state.config.battlenet_client_id.trim().is_empty()
        || state.config.battlenet_client_secret.trim().is_empty()
        || state.config.battlenet_redirect_uri.trim().is_empty();

    if oauth_not_configured {
        return Err(bad_request("Battle.net login is not configured"));
    }
    if repo::has_provider_identity(&state.pool, user_id, "battlenet").await? {
        return Err(bad_request(
            "A Battle.net account is already connected to this profile",
        ));
    }
    let (csrf_state, nonce) = build_oauth_connect_state(&state.config.jwt_secret, user_id)?;
    let url = format!(
        "https://oauth.battle.net/authorize?client_id={}&scope=openid&state={}&redirect_uri={}&response_type=code",
        urlencoding::encode(&state.config.battlenet_client_id),
        urlencoding::encode(&csrf_state),
        urlencoding::encode(&state.config.battlenet_redirect_uri),
    );
    Ok((url, nonce))
}

pub async fn disconnect_battlenet(state: &AppState, user_id: Uuid) -> Result<(), ApiError> {
    if !repo::has_provider_identity(&state.pool, user_id, "battlenet").await? {
        return Err(bad_request("No Battle.net account is connected to this profile"));
    }
    if !repo::user_has_password(&state.pool, user_id).await? {
        return Err(bad_request(
            "Set a password for your account before disconnecting Battle.net, otherwise you will lose access",
        ));
    }
    repo::remove_bnet_identity(&state.pool, user_id).await?;
    repo::unlock_bnet_game_profile(&state.pool, user_id).await?;
    Ok(())
}

fn build_state_with_mode(
    jwt_secret: &str,
    mode: &str,
    user_id: Option<Uuid>,
) -> Result<(String, String), ApiError> {
    let nonce = Uuid::new_v4().simple().to_string();
    let claims = BnetOAuthStateClaims {
        exp: current_unix_timestamp() + OAUTH_STATE_TTL_SECONDS,
        token_type: "bnet_oauth_state".to_string(),
        mode: mode.to_string(),
        user_id: user_id.map(|value| value.to_string()),
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

fn verify_and_parse_oauth_state(jwt_secret: &str, state: &str, nonce: &str) -> Option<OAuthMode> {
    let claims = decode::<BnetOAuthStateClaims>(
        state,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .ok()?
    .claims;

    if claims.token_type != "bnet_oauth_state" {
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
            .and_then(|value| Uuid::parse_str(value).ok())
            .map(OAuthMode::Connect),
        _ => None,
    }
}

fn build_bnet_http_client() -> Result<reqwest::Client, ApiError> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(BNET_HTTP_CONNECT_TIMEOUT_SECONDS))
        .timeout(Duration::from_secs(BNET_HTTP_REQUEST_TIMEOUT_SECONDS))
        .build()
        .map_err(internal_error)
}

async fn exchange_bnet_code(
    state: &AppState,
    code: &str,
) -> Result<(String, String), ApiError> {
    info!("battlenet token exchange started");
    let client = build_bnet_http_client()?;

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
        .map_err(|e| {
            error!(error = %e, "battlenet token exchange request failed");
            internal_error(format!("BNet token exchange failed: {e}"))
        })?;

    if !token_resp.status().is_success() {
        warn!(status = %token_resp.status(), "battlenet token exchange returned non-success status");
        return Err(bad_request("Battle.net authentication failed"));
    }

    let token_data: BnetTokenResponse = token_resp
        .json()
        .await
        .map_err(|e| {
            error!(error = %e, "failed to parse battlenet token response");
            internal_error(format!("Failed to parse BNet token response: {e}"))
        })?;

    info!("battlenet token exchange succeeded");

    let userinfo_resp = client
        .get("https://oauth.battle.net/userinfo")
        .bearer_auth(&token_data.access_token)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "battlenet userinfo request failed");
            internal_error(format!("BNet userinfo request failed: {e}"))
        })?;

    if !userinfo_resp.status().is_success() {
        warn!(status = %userinfo_resp.status(), "battlenet userinfo returned non-success status");
        return Err(internal_error("Failed to fetch Battle.net user info"));
    }

    let userinfo: BnetUserInfo = userinfo_resp
        .json()
        .await
        .map_err(|e| {
            error!(error = %e, "failed to parse battlenet userinfo response");
            internal_error(format!("Failed to parse BNet userinfo: {e}"))
        })?;

    let battletag = userinfo
        .battletag
        .unwrap_or_else(|| format!("Player#{}", &userinfo.sub[..6.min(userinfo.sub.len())]));

    info!("battlenet userinfo resolved");

    Ok((userinfo.sub, battletag))
}

async fn upsert_or_create_bnet_user(
    state: &AppState,
    sub: &str,
    battletag: &str,
    email: &str,
) -> Result<Uuid, ApiError> {
    let user_id = match repo::find_user_id_by_bnet_sub(&state.pool, sub).await? {
        Some(id) => {
            info!(%id, "battlenet login matched existing identity");
            repo::ensure_bnet_identity(&state.pool, id, sub).await?;
            repo::upsert_bnet_game_profile(&state.pool, id, sub, battletag).await?;
            id
        }
        None => {
            if repo::find_user_login_by_email(&state.pool, email).await?.is_some() {
                return Err(bad_request(
                    "An account with this email already exists. Please log in with your email and password, then connect Battle.net from your profile settings.",
                ));
            }
            let new_id = Uuid::new_v4();
            let (base_username, display_name) = username_from_battletag(battletag);
            let username = resolve_unique_username(&state.pool, &base_username).await?;
            let avatar_url = crate::features::users::models::random_preset_avatar();
            repo::insert_bnet_user(&state.pool, new_id, email, &username, &display_name, avatar_url).await?;
            repo::ensure_bnet_identity(&state.pool, new_id, sub).await?;
            repo::insert_default_role(&state.pool, new_id).await?;
            repo::upsert_bnet_game_profile(&state.pool, new_id, sub, battletag).await?;
            info!(%new_id, "battlenet login created new user");
            new_id
        }
    };

    Ok(user_id)
}

async fn handle_bnet_connect(
    state: &AppState,
    user_id: Uuid,
    sub: &str,
    battletag: &str,
) -> Result<(), ApiError> {
    info!(%user_id, "handling battlenet connect");
    // Only block if the sub is *actively* linked to a different user.
    // A disconnected sub is fair game for any user to claim.
    if let Some(existing_id) = repo::find_user_id_by_bnet_sub(&state.pool, sub).await? {
        if existing_id != user_id {
            warn!(%user_id, %existing_id, "battlenet connect rejected: identity linked to another user");
            return Err(bad_request(
                "This Battle.net account is already linked to another profile",
            ));
        }
        repo::ensure_bnet_identity(&state.pool, user_id, sub).await?;
        info!(%user_id, "battlenet connect re-linked existing identity");
        repo::upsert_bnet_game_profile(&state.pool, user_id, sub, battletag).await?;
        return Ok(());
    }

    repo::ensure_bnet_identity(&state.pool, user_id, sub).await?;
    repo::upsert_bnet_game_profile(&state.pool, user_id, sub, battletag).await?;
    info!(%user_id, "battlenet connect linked new identity");
    Ok(())
}

fn build_bnet_pending_signup_token(
    jwt_secret: &str,
    sub: &str,
    battletag: &str,
) -> Result<String, ApiError> {
    let now = current_unix_timestamp();
    let claims = BnetPendingSignupClaims {
        sub: sub.to_string(),
        battletag: battletag.to_string(),
        exp: now + BNET_PENDING_SIGNUP_TTL_SECONDS,
        token_type: "bnet_pending_signup".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(internal_error)
}

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
    let slug = slug[..slug.len().min(20)].to_string();
    let candidate = if discriminator.is_empty() {
        slug.clone()
    } else {
        let full = format!("{}{}", slug, discriminator);
        full[..full.len().min(24)].to_string()
    };

    (candidate, name_part.to_string())
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
        .map(|duration| duration.as_secs() as usize)
        .unwrap_or(0)
}
