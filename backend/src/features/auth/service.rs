use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::http::{header::AUTHORIZATION, HeaderMap};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

    verify_access_token_str(state, token)
}

pub fn verify_access_token_str(state: &AppState, token: &str) -> Result<Uuid, ApiError> {
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
        can_edit_battletag: !row.has_battlenet_identity,
        has_password: row.has_password,
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

pub(crate) async fn issue_auth_response(
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
