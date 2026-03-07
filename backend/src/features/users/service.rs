use uuid::Uuid;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use crate::{
    app::state::AppState,
    shared::{
        errors::{bad_request, forbidden, not_found, ApiError},
        models::{AuthUser, UpdateUserProfileInput},
    },
};

use super::repo;

pub async fn get_user_profile_public(
    state: &AppState,
    user_id: Uuid,
) -> Result<AuthUser, ApiError> {
    let Some((id, email, display_name, role, battletag, rank_tank, rank_dps, rank_support, is_active)) = repo::find_user_profile_by_id(&state.pool, user_id).await? else {
        return Err(not_found("User not found"));
    };

    if !is_active {
        return Err(not_found("User not found"));
    }

    let has_battlenet_identity = repo::has_provider_identity(&state.pool, user_id, "battlenet").await?;

    Ok(AuthUser {
        id,
        email,
        display_name,
        role,
        battletag,
        rank_tank,
        rank_dps,
        rank_support,
        can_edit_battletag: !has_battlenet_identity,
    })
}

pub async fn update_user_profile_for_user(
    state: &AppState,
    authenticated_user_id: Uuid,
    target_user_id: Uuid,
    payload: UpdateUserProfileInput,
) -> Result<AuthUser, ApiError> {
    if authenticated_user_id != target_user_id {
        return Err(forbidden("You can only edit your own profile"));
    }

    let display_name = payload.display_name.trim();
    if display_name.is_empty() {
        return Err(bad_request("Display name is required"));
    }

    let email = normalize_email(&payload.email);
    if email.is_empty() || !email.contains('@') {
        return Err(bad_request("A valid email is required"));
    }

    if repo::email_exists_for_other_user(&state.pool, target_user_id, &email).await? {
        return Err(bad_request("Email is already registered"));
    }

    let current_profile = get_user_profile_public(state, target_user_id).await?;

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

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_rank(role: &str, rank: &str) -> Result<(), ApiError> {
    if crate::shared::models::OVERWATCH_RANKS.contains(&rank) {
        return Ok(());
    }

    Err(bad_request(&format!("Invalid {} rank", role)))
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| bad_request("Failed to hash password"))
}
