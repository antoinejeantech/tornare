use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::shared::errors::internal_error;

pub async fn find_user_login_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(Uuid, String, Option<String>, String, bool)>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id, email, password_hash, display_name, is_active FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|r| {
        (
            r.get("id"),
            r.get("email"),
            r.get("password_hash"),
            r.get("display_name"),
            r.get("is_active"),
        )
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

pub async fn insert_user(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
    password_hash: &str,
    display_name: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("INSERT INTO users (id, email, password_hash, display_name) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(email)
        .bind(password_hash)
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
) -> Result<Option<(Uuid, String, String, bool)>, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id, email, display_name, is_active FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.map(|r| (r.get("id"), r.get("email"), r.get("display_name"), r.get("is_active"))))
}

pub async fn find_active_session_by_hash(
    pool: &PgPool,
    refresh_hash: &str,
) -> Result<Option<(Uuid, Uuid)>, crate::shared::errors::ApiError> {
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

    Ok(row.map(|r| (r.get("id"), r.get("user_id"))))
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
