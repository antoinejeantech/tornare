use sqlx::Row;
use uuid::Uuid;

use crate::{
    app::state::AppState,
    shared::errors::{forbidden, internal_error, ApiError},
};

pub async fn require_event_view_access(
    state: &AppState,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<String, ApiError> {
    let row = sqlx::query("SELECT role FROM event_memberships WHERE event_id = $1 AND user_id = $2")
        .bind(event_id)
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    let Some(row) = row else {
        return Err(forbidden("You do not have access to this event"));
    };

    Ok(row.get("role"))
}

pub async fn require_event_manage_access(
    state: &AppState,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<(), ApiError> {
    let role = require_event_view_access(state, event_id, user_id).await?;
    if role == "owner" || role == "manager" {
        return Ok(());
    }

    Err(forbidden("You do not have permission to modify this event"))
}

pub async fn require_event_owner_access(
    state: &AppState,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<(), ApiError> {
    let role = require_event_view_access(state, event_id, user_id).await?;
    if role == "owner" {
        return Ok(());
    }

    Err(forbidden("Only event owners can perform this action"))
}

pub async fn require_app_admin(state: &AppState, user_id: Uuid) -> Result<(), ApiError> {
    let row = sqlx::query("SELECT id FROM user_roles WHERE user_id = $1 AND role = 'admin'")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if row.is_some() {
        return Ok(());
    }

    Err(forbidden("Only app admins can perform this action"))
}
