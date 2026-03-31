use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::shared::errors::internal_error;

// ---------------------------------------------------------------------------
// Named row type (replaces anonymous 10-tuple at DB boundary)
// ---------------------------------------------------------------------------

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
                    (u.password_hash IS NOT NULL) AS has_password
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
    }))
}

pub async fn email_exists_for_other_user(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id FROM users WHERE email = $1 AND id <> $2")
        .bind(email)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.is_some())
}

pub async fn username_exists_for_other_user(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id FROM users WHERE username = $1 AND id <> $2")
        .bind(username)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.is_some())
}

pub async fn update_user_profile_fields(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
    display_name: &str,
    email: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE users
         SET username = $1,
             display_name = $2,
             email = $3,
             updated_at = NOW()
         WHERE id = $4",
    )
        .bind(username)
        .bind(display_name)
        .bind(email)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub async fn update_local_identity_email(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE auth_identities
         SET provider_user_id = $1,
             email_from_provider = $1
         WHERE user_id = $2 AND provider = 'local'",
    )
    .bind(email)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn update_user_password_hash(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE users
         SET password_hash = $1,
             updated_at = NOW()
         WHERE id = $2",
    )
        .bind(password_hash)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub async fn upsert_overwatch_profile(
    pool: &PgPool,
    user_id: Uuid,
    battletag: Option<&str>,
    rank_tank: &str,
    rank_dps: &str,
    rank_support: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    let game_profile_id: Uuid = sqlx::query_scalar(
        "INSERT INTO user_game_profiles (id, user_id, game_code, handle, provider, provider_user_id, is_handle_locked)
         VALUES ($1, $2, 'overwatch', $3, 'manual', NULL, false)
         ON CONFLICT (user_id, game_code)
         DO UPDATE SET handle = $3, updated_at = NOW()
         RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(battletag)
    .fetch_one(pool)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO overwatch_profiles (id, user_game_profile_id, rank_tank, rank_dps, rank_support)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_game_profile_id)
         DO UPDATE SET rank_tank = $3, rank_dps = $4, rank_support = $5, updated_at = NOW()",
    )
    .bind(Uuid::new_v4())
    .bind(game_profile_id)
    .bind(rank_tank)
    .bind(rank_dps)
    .bind(rank_support)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn delete_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;
    Ok(result.rows_affected() > 0)
}

pub async fn search_users(
    pool: &PgPool,
    query: &str,
) -> Result<Vec<(Uuid, String, String)>, crate::shared::errors::ApiError> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, username, display_name
         FROM users
         WHERE is_active = true
           AND (username ILIKE $1 OR display_name ILIKE $1)
         ORDER BY
             CASE WHEN username ILIKE $2 THEN 0
                  WHEN display_name ILIKE $2 THEN 1
                  ELSE 2
             END,
             display_name
         LIMIT 8",
    )
    .bind(&pattern)
    .bind(format!("{}%", query))
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    Ok(rows
        .into_iter()
        .map(|r| (r.get("id"), r.get("username"), r.get("display_name")))
        .collect())
}
