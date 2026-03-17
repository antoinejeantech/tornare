use sqlx::Row;
use uuid::Uuid;

use crate::{
    app::state::AppState,
    shared::errors::{forbidden, internal_error, ApiError},
};

async fn has_any_app_role(
    state: &AppState,
    user_id: Uuid,
    roles: &[&str],
) -> Result<bool, ApiError> {
    let roles: Vec<String> = roles.iter().map(|role| (*role).to_string()).collect();

    let has_role = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM user_roles
            WHERE user_id = $1 AND role = ANY($2)
        )",
    )
    .bind(user_id)
    .bind(&roles)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(has_role)
}

pub async fn has_global_event_owner_access(
    state: &AppState,
    user_id: Uuid,
) -> Result<bool, ApiError> {
    has_any_app_role(state, user_id, &["admin", "moderator"]).await
}

pub async fn require_event_admin_access(state: &AppState, user_id: Uuid) -> Result<(), ApiError> {
    if has_global_event_owner_access(state, user_id).await? {
        return Ok(());
    }

    Err(forbidden("Only app admins and moderators can perform this action"))
}

pub async fn require_event_view_access(
    state: &AppState,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<String, ApiError> {
    if has_global_event_owner_access(state, user_id).await? {
        return Ok("owner".to_string());
    }

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

/// Returns `true` if the user is a literal event member with "owner" role,
/// or `false` if access is granted via global admin/moderator elevation.
pub async fn require_event_owner_access(
    state: &AppState,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<bool, ApiError> {
    let has_global_access = has_global_event_owner_access(state, user_id).await?;
    if has_global_access {
        return Ok(false);
    }

    let role = require_event_view_access(state, event_id, user_id).await?;
    if role == "owner" {
        return Ok(true);
    }

    Err(forbidden("Only event owners can perform this action"))
}
