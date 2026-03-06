use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::shared::errors::internal_error;

pub async fn list_visible_match_ids(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Uuid>, crate::shared::errors::ApiError> {
    let rows = sqlx::query(
        "SELECT em.id
         FROM event_matches em
         INNER JOIN event_memberships m ON m.event_id = em.event_id
         WHERE m.user_id = $1
         ORDER BY em.id DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    Ok(rows.into_iter().map(|row| row.get("id")).collect())
}

pub async fn get_match_event_id(
    pool: &PgPool,
    match_id: Uuid,
) -> Result<Option<Uuid>, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT event_id FROM event_matches WHERE id = $1")
        .bind(match_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.map(|r| r.get("event_id")))
}

pub async fn delete_match_by_id(
    pool: &PgPool,
    match_id: Uuid,
) -> Result<u64, crate::shared::errors::ApiError> {
    let result = sqlx::query("DELETE FROM event_matches WHERE id = $1")
        .bind(match_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(result.rows_affected())
}
