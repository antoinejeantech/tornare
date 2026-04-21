use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::auth::require_authenticated_user_id,
    shared::{
        errors::ApiResult,
        models::MessageResponse,
    },
};

use super::{
    models::{
        AddGuildMemberInput, BotInviteUrl, DiscordGuild, GuildMember,
        SetMentionRolesInput, ToggleAnnouncementsInput, UpsertGuildInput,
    },
    service,
};

/// GET /api/discord/guilds
pub async fn get_my_guilds(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Vec<DiscordGuild>> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::get_my_guilds(&state, user_id).await.map(Json)
}

/// PUT /api/discord/guild
pub async fn upsert_my_guild(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpsertGuildInput>,
) -> ApiResult<DiscordGuild> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::upsert_my_guild(&state, user_id, payload)
        .await
        .map(Json)
}

/// PATCH /api/discord/guild/:guild_id/announcements
pub async fn toggle_announcements(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    Json(payload): Json<ToggleAnnouncementsInput>,
) -> ApiResult<DiscordGuild> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::toggle_announcements(&state, user_id, &guild_id, payload)
        .await
        .map(Json)
}

/// DELETE /api/discord/guild/:guild_id
pub async fn delete_my_guild(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<String>,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::delete_my_guild(&state, user_id, &guild_id).await?;
    Ok(Json(MessageResponse { message: "Guild deleted".into() }))
}

/// GET /api/discord/guild/:guild_id/members
pub async fn list_guild_members(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<String>,
) -> ApiResult<Vec<GuildMember>> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::list_guild_members(&state, user_id, &guild_id)
        .await
        .map(Json)
}

/// POST /api/discord/guild/:guild_id/members
pub async fn add_guild_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    Json(payload): Json<AddGuildMemberInput>,
) -> ApiResult<Vec<GuildMember>> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::add_guild_member(&state, user_id, &guild_id, payload)
        .await
        .map(Json)
}

/// DELETE /api/discord/guild/:guild_id/members/:user_id
pub async fn remove_guild_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((guild_id, target_user_id)): Path<(String, Uuid)>,
) -> ApiResult<Vec<GuildMember>> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::remove_guild_member(&state, user_id, &guild_id, target_user_id)
        .await
        .map(Json)
}

/// GET /api/discord/invite
pub async fn get_bot_invite_url(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<BotInviteUrl> {
    require_authenticated_user_id(&state, &headers)?;
    Ok(Json(service::get_bot_invite_url(&state)))
}

/// PATCH /api/discord/guild/:guild_id/mention-roles
pub async fn set_mention_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    Json(payload): Json<SetMentionRolesInput>,
) -> ApiResult<DiscordGuild> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::set_mention_roles(&state, user_id, &guild_id, payload)
        .await
        .map(Json)
}
