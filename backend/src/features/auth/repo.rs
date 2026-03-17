use sqlx::{PgPool, Row};
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
}

/// Active session looked up by refresh-token hash.
pub struct ActiveSessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
}

/// Login credentials looked up by email.
pub struct UserLoginRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub display_name: String,
    pub is_active: bool,
}

pub async fn find_user_login_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserLoginRow>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id, email, password_hash, display_name, is_active FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|r| UserLoginRow {
        id: r.get("id"),
        email: r.get("email"),
        password_hash: r.get("password_hash"),
        display_name: r.get("display_name"),
        is_active: r.get("is_active"),
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
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("INSERT INTO users (id, email, password_hash, username, display_name) VALUES ($1, $2, $3, $4, $5)")
        .bind(user_id)
        .bind(email)
        .bind(password_hash)
        .bind(username)
        .bind(display_name)
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
                        u.is_active
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
    }))
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
