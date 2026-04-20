use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        auth::models::AuthUser,
        events::repo as events_repo,
        users::models::{ParticipatedEventSummary, UpdateUserProfileInput, UserSearchResult, OVERWATCH_RANKS},
    },
    shared::{
        crypto::hash_password,
        errors::{bad_request, forbidden, not_found, ApiError},
        validation::{normalize_email, normalize_username},
    },
};

use super::repo;

pub async fn get_user_profile_public(
    state: &AppState,
    user_id: Uuid,
) -> Result<AuthUser, ApiError> {
    let Some(row) = repo::find_user_profile_by_id(&state.pool, user_id).await? else {
        return Err(not_found("User not found"));
    };

    if !row.is_active {
        return Err(not_found("User not found"));
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

pub async fn update_user_profile_for_user(
    state: &AppState,
    authenticated_user_id: Uuid,
    target_user_id: Uuid,
    payload: UpdateUserProfileInput,
) -> Result<AuthUser, ApiError> {
    if authenticated_user_id != target_user_id {
        let Some(actor) = repo::find_user_profile_by_id(&state.pool, authenticated_user_id).await? else {
            return Err(forbidden("You do not have permission to edit this profile"));
        };

        if !actor.is_active || !actor.role.eq_ignore_ascii_case("admin") {
            return Err(forbidden("You can only edit your own profile unless you are an admin"));
        }
    }

    let display_name = payload.display_name.trim();
    if display_name.is_empty() {
        return Err(bad_request("Display name is required"));
    }

    let username = normalize_username(&payload.username)?;

    let email = normalize_email(&payload.email)?;

    if repo::email_exists_for_other_user(&state.pool, target_user_id, &email).await? {
        return Err(bad_request("Email is already registered"));
    }

    if repo::username_exists_for_other_user(&state.pool, target_user_id, &username).await? {
        return Err(bad_request("Username is already taken"));
    }

    let current_profile = get_user_profile_public(state, target_user_id).await?;

    // Discord-linked accounts use Discord as the source of truth for email.
    if current_profile.has_discord_identity && email != current_profile.email {
        return Err(bad_request(
            "Email cannot be changed for accounts linked with Discord. Disconnect Discord first.",
        ));
    }

    let battletag = payload
        .battletag
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let requested = battletag.as_deref().unwrap_or("");
    let current = current_profile.battletag.as_deref().unwrap_or("");
    if requested != current {
        return Err(forbidden("Battletag is read-only until Battle.net OAuth is available"));
    }

    validate_rank("tank", &payload.rank_tank)?;
    validate_rank("dps", &payload.rank_dps)?;
    validate_rank("support", &payload.rank_support)?;

    repo::update_user_profile_fields(
        &state.pool,
        target_user_id,
        &username,
        display_name,
        &email,
    )
    .await?;
    repo::upsert_overwatch_profile(
        &state.pool,
        target_user_id,
        battletag.as_deref(),
        &payload.rank_tank,
        &payload.rank_dps,
        &payload.rank_support,
    )
    .await?;
    repo::update_local_identity_email(&state.pool, target_user_id, &email).await?;

    let next_password = payload
        .new_password
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    let next_password_confirm = payload
        .new_password_confirm
        .as_deref()
        .map(str::trim)
        .unwrap_or("");

    if !next_password.is_empty() || !next_password_confirm.is_empty() {
        if next_password.len() < 8 {
            return Err(bad_request("Password must be at least 8 characters long"));
        }

        if next_password != next_password_confirm {
            return Err(bad_request("Passwords do not match"));
        }

        let password_hash = hash_password(next_password)?;
        repo::update_user_password_hash(&state.pool, target_user_id, &password_hash).await?;
    }

    get_user_profile_public(state, target_user_id).await
}

pub async fn update_user_avatar(
    state: &AppState,
    authenticated_user_id: Uuid,
    target_user_id: Uuid,
    avatar_url: Option<&str>,
) -> Result<AuthUser, ApiError> {
    use crate::features::users::models::ALLOWED_PRESET_AVATARS;

    if authenticated_user_id != target_user_id {
        let Some(actor) = repo::find_user_profile_by_id(&state.pool, authenticated_user_id).await? else {
            return Err(forbidden("You do not have permission to edit this profile"));
        };
        if !actor.is_active || !actor.role.eq_ignore_ascii_case("admin") {
            return Err(forbidden("You can only edit your own profile unless you are an admin"));
        }
    }

    if let Some(url) = avatar_url {
        if !ALLOWED_PRESET_AVATARS.contains(&url) {
            return Err(bad_request("Invalid avatar selection"));
        }
    }

    repo::update_user_avatar_url(&state.pool, target_user_id, avatar_url).await?;
    get_user_profile_public(state, target_user_id).await
}

pub async fn delete_user_account(
    state: &AppState,
    authenticated_user_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), ApiError> {
    if authenticated_user_id == target_user_id {
        return Err(forbidden("Admins cannot delete their own account"));
    }

    let Some(actor) = repo::find_user_profile_by_id(&state.pool, authenticated_user_id).await? else {
        return Err(forbidden("You do not have permission to delete this account"));
    };

    if !actor.is_active || !actor.role.eq_ignore_ascii_case("admin") {
        return Err(forbidden("Only admins can delete accounts"));
    }

    let deleted = repo::delete_user_by_id(&state.pool, target_user_id).await?;
    if !deleted {
        return Err(not_found("User not found"));
    }
    Ok(())
}

fn validate_rank(role: &str, rank: &str) -> Result<(), ApiError> {
    if OVERWATCH_RANKS.contains(&rank) {
        return Ok(());
    }

    Err(bad_request(&format!("Invalid {} rank", role)))
}

pub async fn search_users(
    state: &AppState,
    query: &str,
) -> Result<Vec<UserSearchResult>, ApiError> {
    if query.is_empty() {
        return Ok(vec![]);
    }

    let rows = repo::search_users(&state.pool, query).await?;

    Ok(rows
        .into_iter()
        .map(|(id, username, display_name)| UserSearchResult { id, username, display_name })
        .collect())
}

pub async fn get_participated_events(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<ParticipatedEventSummary>, ApiError> {
    let rows = events_repo::list_participated_events(&state.pool, user_id, 10).await?;

    Ok(rows
        .into_iter()
        .map(|r| ParticipatedEventSummary {
            id: r.id,
            name: r.name,
            start_date: r.start_date.map(|dt| {
                dt.format(&time::format_description::well_known::Rfc3339)
                    .expect("start_date retrieved from the DB must always be RFC3339-formattable")
            }),
            event_type: r.event_type.as_db_value().to_string(),
            format: r.format.as_db_value().to_string(),
            status: r.status.as_db_value().to_string(),
        })
        .collect())
}
