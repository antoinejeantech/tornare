use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::shared::errors::internal_error;

// ---------------------------------------------------------------------------
// Named row types (replace anonymous tuples at DB boundary)
// ---------------------------------------------------------------------------

/// Profile data loaded from the DB for authentication / user-info endpoints.
pub struct UserProfileRow {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub battletag: Option<String>,
    pub rank_tank: String,
    pub rank_dps: String,
    pub rank_support: String,
    pub is_active: bool,
    pub has_battlenet_identity: bool,
    pub has_discord_identity: bool,
    pub discord_username: Option<String>,
    pub has_password: bool,
    pub avatar_url: Option<String>,
}

/// Active session looked up by refresh-token hash.
pub struct ActiveSessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
}

/// Login credentials looked up by email.
pub struct UserLoginRow {
    pub id: Uuid,
    pub password_hash: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
}

/// A verification token row from `email_verification_tokens`.
pub struct VerificationTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: OffsetDateTime,
    pub used_at: Option<OffsetDateTime>,
}

pub async fn find_user_login_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserLoginRow>, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id, password_hash, is_active, email_verified FROM users WHERE email = $1")
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|r| UserLoginRow {
        id: r.get("id"),
        password_hash: r.get("password_hash"),
        is_active: r.get("is_active"),
        email_verified: r.get("email_verified"),
    }))
}

pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;
    Ok(row.is_some())
}

pub async fn username_exists(pool: &PgPool, username: &str) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;
    Ok(row.is_some())
}

pub async fn insert_user(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
    password_hash: &str,
    username: &str,
    display_name: &str,
    avatar_url: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("INSERT INTO users (id, email, password_hash, username, display_name, avatar_url) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(user_id)
        .bind(email)
        .bind(password_hash)
        .bind(username)
        .bind(display_name)
        .bind(avatar_url)
        .execute(pool)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn insert_local_identity(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id, email_from_provider)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind("local")
    .bind(email)
    .bind(email)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn insert_default_role(pool: &PgPool, user_id: Uuid) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("INSERT INTO user_roles (id, user_id, role) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind("user")
        .execute(pool)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn find_user_profile_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<UserProfileRow>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
                "SELECT
                        u.id,
                        u.email,
                        u.username,
                        u.display_name,
                    COALESCE(
                        (
                            SELECT ur.role
                            FROM user_roles ur
                            WHERE ur.user_id = u.id
                            ORDER BY
                                CASE ur.role
                                    WHEN 'admin' THEN 0
                                    WHEN 'moderator' THEN 1
                                    ELSE 2
                                END,
                                ur.role
                            LIMIT 1
                        ),
                        'user'
                    ) AS role,
                    ugp.handle AS battletag,
                    COALESCE(op.rank_tank, 'Unranked') AS rank_tank,
                    COALESCE(op.rank_dps, 'Unranked') AS rank_dps,
                    COALESCE(op.rank_support, 'Unranked') AS rank_support,
                    u.is_active,
                    EXISTS(
                        SELECT 1
                        FROM auth_identities ai
                        WHERE ai.user_id = u.id
                          AND ai.provider = 'battlenet'
                    ) AS has_battlenet_identity,
                    EXISTS(
                        SELECT 1
                        FROM auth_identities ai
                        WHERE ai.user_id = u.id
                          AND ai.provider = 'discord'
                    ) AS has_discord_identity,
                    (SELECT ai.provider_username
                     FROM auth_identities ai
                     WHERE ai.user_id = u.id
                       AND ai.provider = 'discord'
                     LIMIT 1) AS discord_username,
                    (u.password_hash IS NOT NULL) AS has_password,
                    u.avatar_url
                 FROM users u
                 LEFT JOIN user_game_profiles ugp
                     ON ugp.user_id = u.id
                    AND ugp.game_code = 'overwatch'
                 LEFT JOIN overwatch_profiles op
                     ON op.user_game_profile_id = ugp.id
                 WHERE u.id = $1",
    )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.map(|r| UserProfileRow {
        id: r.get("id"),
        email: r.get("email"),
        username: r.get("username"),
        display_name: r.get("display_name"),
        role: r.get("role"),
        battletag: r.get("battletag"),
        rank_tank: r.get("rank_tank"),
        rank_dps: r.get("rank_dps"),
        rank_support: r.get("rank_support"),
        is_active: r.get("is_active"),
        has_battlenet_identity: r.get("has_battlenet_identity"),
        has_discord_identity: r.get("has_discord_identity"),
        discord_username: r.get("discord_username"),
        has_password: r.get("has_password"),
        avatar_url: r.get("avatar_url"),
    }))
}

pub async fn user_has_password(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id FROM users WHERE id = $1 AND password_hash IS NOT NULL",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;
    Ok(row.is_some())
}

pub async fn has_provider_identity(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id FROM auth_identities WHERE user_id = $1 AND provider = $2")
        .bind(user_id)
        .bind(provider)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.is_some())
}

pub async fn find_active_session_by_hash(
    pool: &PgPool,
    refresh_hash: &str,
) -> Result<Option<ActiveSessionRow>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id, user_id
         FROM auth_sessions
         WHERE refresh_token_hash = $1
           AND revoked_at IS NULL
           AND expires_at > NOW()",
    )
    .bind(refresh_hash)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|r| ActiveSessionRow {
        id: r.get("id"),
        user_id: r.get("user_id"),
    }))
}

pub async fn rotate_session(
    pool: &PgPool,
    session_id: Uuid,
    refresh_hash: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE auth_sessions
         SET refresh_token_hash = $1,
             expires_at = NOW() + interval '30 days',
             revoked_at = NULL
         WHERE id = $2",
    )
    .bind(refresh_hash)
    .bind(session_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    refresh_hash: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO auth_sessions (id, user_id, refresh_token_hash, expires_at)
         VALUES ($1, $2, $3, NOW() + interval '30 days')",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(refresh_hash)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn revoke_session_by_hash(
    pool: &PgPool,
    refresh_hash: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("UPDATE auth_sessions SET revoked_at = NOW() WHERE refresh_token_hash = $1")
        .bind(refresh_hash)
        .execute(pool)
        .await
        .map_err(internal_error)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Battle.net OAuth helpers
// ---------------------------------------------------------------------------

pub async fn find_user_id_by_bnet_sub(
    pool: &PgPool,
    sub: &str,
) -> Result<Option<Uuid>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT user_id
         FROM auth_identities
         WHERE provider = 'battlenet'
           AND provider_user_id = $1
         LIMIT 1",
    )
    .bind(sub)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|r| r.get("user_id")))
}

// find_user_id_by_bnet_sub covers both login and connect checks

pub async fn insert_bnet_user(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
    username: &str,
    display_name: &str,
    avatar_url: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, avatar_url) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(email)
    .bind(username)
    .bind(display_name)
    .bind(avatar_url)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn ensure_bnet_identity(
    pool: &PgPool,
    user_id: Uuid,
    sub: &str,
    battletag: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id, provider_username)
         VALUES ($1, $2, 'battlenet', $3, $4)
         ON CONFLICT (provider, provider_user_id)
         DO UPDATE SET provider_username = EXCLUDED.provider_username",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(sub)
    .bind(battletag)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn upsert_bnet_game_profile(
    pool: &PgPool,
    user_id: Uuid,
    sub: &str,
    battletag: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO user_game_profiles
             (id, user_id, game_code, handle, provider, provider_user_id, is_handle_locked)
         VALUES ($1, $2, 'overwatch', $3, 'battlenet', $4, true)
         ON CONFLICT (user_id, game_code)
         DO UPDATE SET
             handle           = EXCLUDED.handle,
             provider         = 'battlenet',
             provider_user_id = EXCLUDED.provider_user_id,
             is_handle_locked = true,
             updated_at       = NOW()",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(battletag)
    .bind(sub)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn remove_bnet_identity(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "DELETE FROM auth_identities
         WHERE user_id = $1 AND provider = 'battlenet'",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn unlock_bnet_game_profile(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE user_game_profiles
         SET handle = NULL, provider = 'manual', provider_user_id = NULL, is_handle_locked = false,
             updated_at = NOW()
         WHERE user_id = $1 AND game_code = 'overwatch'",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Discord OAuth helpers
// ---------------------------------------------------------------------------

pub async fn find_user_id_by_discord_sub(
    pool: &PgPool,
    sub: &str,
) -> Result<Option<Uuid>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT user_id
         FROM auth_identities
         WHERE provider = 'discord'
           AND provider_user_id = $1
         LIMIT 1",
    )
    .bind(sub)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|r| r.get("user_id")))
}

pub async fn insert_discord_user(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
    username: &str,
    display_name: &str,
    avatar_url: Option<&str>,
) -> Result<(), crate::shared::errors::ApiError> {
    // Discord provides a pre-verified email — mark verified immediately.
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, avatar_url, email_verified) VALUES ($1, $2, $3, $4, $5, TRUE)",
    )
    .bind(user_id)
    .bind(email)
    .bind(username)
    .bind(display_name)
    .bind(avatar_url)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn update_user_avatar_url(
    pool: &PgPool,
    user_id: Uuid,
    avatar_url: Option<&str>,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
        .bind(avatar_url)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn ensure_discord_identity(
    pool: &PgPool,
    user_id: Uuid,
    sub: &str,
    username: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id, provider_username)
         VALUES ($1, $2, 'discord', $3, $4)
         ON CONFLICT (provider, provider_user_id)
         DO UPDATE SET provider_username = EXCLUDED.provider_username",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(sub)
    .bind(username)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn remove_discord_identity(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "DELETE FROM auth_identities
         WHERE user_id = $1 AND provider = 'discord'",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Email verification tokens
// ---------------------------------------------------------------------------

fn hash_verification_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

/// Creates a new single-use verification token (24 h TTL) and returns the raw token string.
pub async fn create_verification_token(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<String, crate::shared::errors::ApiError> {
    let raw_token = format!("{}.{}", Uuid::new_v4(), Uuid::new_v4());
    let token_hash = hash_verification_token(&raw_token);
    sqlx::query(
        "INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at)
         VALUES ($1, $2, $3, NOW() + INTERVAL '24 hours')",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(&token_hash)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(raw_token)
}

pub async fn find_token_by_hash(
    pool: &PgPool,
    raw_token: &str,
) -> Result<Option<VerificationTokenRow>, crate::shared::errors::ApiError> {
    let token_hash = hash_verification_token(raw_token);
    let row = sqlx::query(
        "SELECT id, user_id, expires_at, used_at
         FROM email_verification_tokens
         WHERE token_hash = $1",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|r| VerificationTokenRow {
        id: r.get("id"),
        user_id: r.get("user_id"),
        expires_at: r.get("expires_at"),
        used_at: r.get("used_at"),
    }))
}

/// Atomically claims a verification token by setting `used_at`, guarded by
/// `used_at IS NULL`. Returns `true` if the token was claimed, `false` if a
/// concurrent request already claimed it.
pub async fn mark_token_used(
    pool: &PgPool,
    token_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let result =
        sqlx::query("UPDATE email_verification_tokens SET used_at = NOW() WHERE id = $1 AND used_at IS NULL")
            .bind(token_id)
            .execute(pool)
            .await
            .map_err(internal_error)?;
    Ok(result.rows_affected() > 0)
}

pub async fn mark_email_verified(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("UPDATE users SET email_verified = TRUE WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;
    Ok(())
}

/// Returns the `created_at` timestamp of the most recent token for this user,
/// used to enforce a per-user resend rate-limit.
pub async fn get_latest_token_created_at(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<OffsetDateTime>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT created_at FROM email_verification_tokens
         WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;
    Ok(row.map(|r| r.get("created_at")))
}

// ---------------------------------------------------------------------------
// Password reset tokens
// ---------------------------------------------------------------------------

/// A password reset token row from `password_reset_tokens`.
pub struct PasswordResetTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: OffsetDateTime,
    pub used_at: Option<OffsetDateTime>,
}

/// Creates a new single-use password reset token (1 h TTL) and returns the raw token string.
pub async fn create_password_reset_token(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<String, crate::shared::errors::ApiError> {
    let raw_token = format!("{}.{}", Uuid::new_v4(), Uuid::new_v4());
    let token_hash = hash_verification_token(&raw_token);
    sqlx::query(
        "INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at)
         VALUES ($1, $2, $3, NOW() + INTERVAL '1 hour')",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(&token_hash)
    .execute(pool)
    .await
    .map_err(internal_error)?;
    Ok(raw_token)
}

pub async fn find_reset_token_by_hash(
    pool: &PgPool,
    raw_token: &str,
) -> Result<Option<PasswordResetTokenRow>, crate::shared::errors::ApiError> {
    let token_hash = hash_verification_token(raw_token);
    let row = sqlx::query(
        "SELECT id, user_id, expires_at, used_at
         FROM password_reset_tokens
         WHERE token_hash = $1",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|r| PasswordResetTokenRow {
        id: r.get("id"),
        user_id: r.get("user_id"),
        expires_at: r.get("expires_at"),
        used_at: r.get("used_at"),
    }))
}

/// Atomically claims a password reset token by setting `used_at`, guarded by
/// `used_at IS NULL`. Returns `true` if the token was claimed, `false` if a
/// concurrent request already claimed it.
pub async fn mark_reset_token_used(
    pool: &PgPool,
    token_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let result =
        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1 AND used_at IS NULL")
            .bind(token_id)
            .execute(pool)
            .await
            .map_err(internal_error)?;
    Ok(result.rows_affected() > 0)
}

/// Returns the `created_at` of the most recent reset token for rate-limiting.
pub async fn get_latest_reset_token_created_at(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<OffsetDateTime>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT created_at FROM password_reset_tokens
         WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;
    Ok(row.map(|r| r.get("created_at")))
}

pub async fn update_user_password(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(password_hash)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;
    Ok(())
}

