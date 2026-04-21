use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::http::{header::AUTHORIZATION, HeaderMap};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::auth::models::{AuthResponse, AuthUser, ForgotPasswordInput, LoginInput, PendingVerificationResponse, RegisterInput, ResetPasswordInput},
    shared::{
        crypto::hash_password,
        errors::{bad_request, email_not_verified, forbidden, internal_error, too_many_requests, unauthorized, ApiError},
        models::MessageResponse,
        validation::{normalize_email, normalize_username},
    },
};

use super::{email as auth_email, repo};

const ACCESS_TOKEN_TTL_SECONDS: usize = 15 * 60;

#[derive(Serialize, Deserialize)]
struct AccessClaims {
    sub: String,
    exp: usize,
    token_type: String,
}

pub async fn register_user(state: &AppState, payload: RegisterInput) -> Result<PendingVerificationResponse, ApiError> {
    validate_register_input(&payload)?;

    let normalized_email = normalize_email(&payload.email)?;
    if repo::email_exists(&state.pool, &normalized_email).await? {
        return Err(bad_request("Email is already registered"));
    }

    let normalized_username = normalize_username(&payload.username)?;
    if repo::username_exists(&state.pool, &normalized_username).await? {
        return Err(bad_request("Username is already taken"));
    }

    let password_hash = hash_password(&payload.password)?;
    let user_id = Uuid::new_v4();
    let avatar_url = crate::features::users::models::random_preset_avatar();

    repo::insert_user(
        &state.pool,
        user_id,
        &normalized_email,
        &password_hash,
        &normalized_username,
        payload.display_name.trim(),
        avatar_url,
    )
    .await?;
    repo::insert_local_identity(&state.pool, user_id, &normalized_email).await?;
    repo::insert_default_role(&state.pool, user_id).await?;

    // Send verification email — best-effort; registration succeeds even if email fails.
    match repo::create_verification_token(&state.pool, user_id).await {
        Ok(raw_token) => {
            if let Err(e) = auth_email::send_verification_email(&state.config, &normalized_email, &raw_token).await {
                tracing::error!("Failed to send verification email for {normalized_email}: {e:?}");
            }
        }
        Err(e) => tracing::error!("Failed to create verification token: {e:?}"),
    }

    Ok(PendingVerificationResponse {
        message: "Account created. Please check your email to verify your account.".to_string(),
    })
}

pub async fn login_user(state: &AppState, payload: LoginInput) -> Result<AuthResponse, ApiError> {
    let normalized_email = normalize_email(&payload.email)?;

    if payload.password.is_empty() {
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

    if !login.email_verified {
        return Err(email_not_verified());
    }

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

/// Like `maybe_authenticated_user_id` but returns `Err(401)` when an
/// `Authorization: Bearer` header is present yet invalid or expired.
/// Use this on endpoints that silently degrade to anonymous access so that
/// the client's automatic token-refresh cycle is triggered correctly.
pub fn strict_maybe_authenticated_user_id(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<Uuid>, ApiError> {
    let Some(auth_header) = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return Ok(None); // no header → anonymous
    };

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| unauthorized("Authorization header must use Bearer token"))?;

    verify_access_token_str(state, token).map(Some)
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
        has_discord_identity: row.has_discord_identity,
        discord_username: row.discord_username,
        avatar_url: row.avatar_url,
    })
}

fn validate_register_input(payload: &RegisterInput) -> Result<(), ApiError> {
    normalize_email(&payload.email)?;

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

/// Consumes a verification token and issues auth tokens.
pub async fn verify_email(state: &AppState, raw_token: &str) -> Result<AuthResponse, ApiError> {
    if raw_token.trim().is_empty() {
        return Err(bad_request("Verification token is required"));
    }

    let Some(token) = repo::find_token_by_hash(&state.pool, raw_token).await? else {
        return Err(bad_request("Invalid or expired verification link"));
    };

    if token.used_at.is_some() {
        return Err(bad_request("This verification link has already been used"));
    }

    if token.expires_at < OffsetDateTime::now_utc() {
        return Err(bad_request(
            "This verification link has expired. Please request a new one.",
        ));
    }

    if !repo::mark_token_used(&state.pool, token.id).await? {
        return Err(bad_request("This verification link has already been used"));
    }
    repo::mark_email_verified(&state.pool, token.user_id).await?;

    let user = get_auth_user_by_id(state, token.user_id).await?;
    issue_auth_response(state, user, None).await
}

/// Sends a fresh verification email, with a 60-second per-user rate limit.
pub async fn resend_verification(
    state: &AppState,
    email: &str,
) -> Result<MessageResponse, ApiError> {
    let normalized_email = normalize_email(email)?;

    // Always return a generic message to avoid account enumeration.
    let Some(login) = repo::find_user_login_by_email(&state.pool, &normalized_email).await? else {
        return Ok(MessageResponse {
            message: "If an account with that email exists, a verification email has been sent."
                .to_string(),
        });
    };

    if login.email_verified {
        return Ok(MessageResponse {
            message: "This email address is already verified.".to_string(),
        });
    }

    const RESEND_COOLDOWN_SECONDS: i64 = 60;
    if let Some(latest) = repo::get_latest_token_created_at(&state.pool, login.id).await? {
        let age = (OffsetDateTime::now_utc() - latest).whole_seconds();
        if age < RESEND_COOLDOWN_SECONDS {
            let wait = RESEND_COOLDOWN_SECONDS - age;
            return Err(too_many_requests(&format!(
                "Please wait {wait} seconds before requesting another verification email"
            )));
        }
    }

    let raw_token = repo::create_verification_token(&state.pool, login.id).await?;
    auth_email::send_verification_email(&state.config, &normalized_email, &raw_token).await?;

    Ok(MessageResponse {
        message: "Verification email sent. Please check your inbox.".to_string(),
    })
}

/// Sends a password reset email. Always returns a generic success message (anti-enumeration).
pub async fn forgot_password(
    state: &AppState,
    payload: ForgotPasswordInput,
) -> Result<MessageResponse, ApiError> {
    let generic_ok = MessageResponse {
        message: "If an account with that email exists, you'll receive a reset link shortly."
            .to_string(),
    };

    let Ok(normalized_email) = normalize_email(&payload.email) else {
        return Ok(generic_ok);
    };

    let Some(login) = repo::find_user_login_by_email(&state.pool, &normalized_email).await? else {
        return Ok(generic_ok);
    };

    const RESET_COOLDOWN_SECONDS: i64 = 60;
    if let Some(latest) =
        repo::get_latest_reset_token_created_at(&state.pool, login.id).await?
    {
        let age = (OffsetDateTime::now_utc() - latest).whole_seconds();
        if age < RESET_COOLDOWN_SECONDS {
            let wait = RESET_COOLDOWN_SECONDS - age;
            return Err(too_many_requests(&format!(
                "Please wait {wait} seconds before requesting another reset email"
            )));
        }
    }

    let raw_token = repo::create_password_reset_token(&state.pool, login.id).await?;
    if let Err(e) =
        auth_email::send_password_reset_email(&state.config, &normalized_email, &raw_token).await
    {
        tracing::error!(?e, "Failed to send password reset email");
    }

    Ok(generic_ok)
}

/// Validates a reset token and updates the user's password.
pub async fn reset_password(
    state: &AppState,
    payload: ResetPasswordInput,
) -> Result<MessageResponse, ApiError> {
    if payload.new_password.len() < 8 {
        return Err(bad_request("Password must be at least 8 characters"));
    }
    if payload.new_password != payload.new_password_confirm {
        return Err(bad_request("Passwords do not match"));
    }

    let Some(token) = repo::find_reset_token_by_hash(&state.pool, &payload.token).await? else {
        return Err(bad_request("Invalid or expired reset link"));
    };

    if token.used_at.is_some() {
        return Err(bad_request("This reset link has already been used"));
    }

    if token.expires_at < OffsetDateTime::now_utc() {
        return Err(bad_request(
            "This reset link has expired. Please request a new one.",
        ));
    }

    let password_hash = hash_password(&payload.new_password)?;
    // Claim the token atomically before updating the password so that
    // concurrent reset requests with the same token cannot both succeed.
    if !repo::mark_reset_token_used(&state.pool, token.id).await? {
        return Err(bad_request("This reset link has already been used"));
    }
    repo::update_user_password(&state.pool, token.user_id, &password_hash).await?;
    // Resetting password also proves email ownership.
    repo::mark_email_verified(&state.pool, token.user_id).await?;

    Ok(MessageResponse {
        message: "Password updated. You can now sign in.".to_string(),
    })
}

