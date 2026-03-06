use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::auth::{maybe_authenticated_user_id, require_authenticated_user_id},
    shared::{
        errors::ApiResult,
        models::{
            AddPlayerInput, AssignEventPlayerTeamInput, CreateEventInput, CreateEventMatchInput,
            CreateEventSignupRequestInput, CreateEventTeamInput, Event, EventSignupLinkResponse,
            EventSignupRequest, Match, MessageResponse, PublicEventSignupInfo, SetMatchupInput,
            UpdateEventInput, UpdateEventPlayerInput, UpdateEventTeamInput,
        },
    },
};

use super::service;

pub async fn list_events(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Vec<Event>> {
    let viewer_user_id = maybe_authenticated_user_id(&state, &headers);
    service::list_events_public(&state, viewer_user_id)
        .await
        .map(Json)
}

pub async fn get_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    let viewer_user_id = maybe_authenticated_user_id(&state, &headers);
    service::get_event_public(&state, event_id, viewer_user_id)
        .await
        .map(Json)
}

pub async fn create_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEventInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::create_event_for_user(&state, user_id, payload)
        .await
        .map(Json)
}

pub async fn update_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateEventInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::update_event_for_user(&state, user_id, event_id, payload)
        .await
        .map(Json)
}

pub async fn delete_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::delete_event_for_user(&state, user_id, event_id)
        .await
        .map(Json)
}

pub async fn create_event_match(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEventMatchInput>,
) -> ApiResult<Match> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::create_event_match_for_user(&state, user_id, event_id, payload)
        .await
        .map(Json)
}

pub async fn add_event_player(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AddPlayerInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::add_event_player_for_user(&state, user_id, event_id, payload)
        .await
        .map(Json)
}

pub async fn delete_event_player(
    Path((event_id, player_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::delete_event_player_for_user(&state, user_id, event_id, player_id)
        .await
        .map(Json)
}

pub async fn update_event_player(
    Path((event_id, player_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateEventPlayerInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::update_event_player_for_user(&state, user_id, event_id, player_id, payload)
        .await
        .map(Json)
}

pub async fn create_event_team(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEventTeamInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::create_event_team_for_user(&state, user_id, event_id, payload)
        .await
        .map(Json)
}

pub async fn delete_event_team(
    Path((event_id, team_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::delete_event_team_for_user(&state, user_id, event_id, team_id)
        .await
        .map(Json)
}

pub async fn update_event_team(
    Path((event_id, team_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateEventTeamInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::update_event_team_for_user(&state, user_id, event_id, team_id, payload)
        .await
        .map(Json)
}

pub async fn assign_event_player_team(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AssignEventPlayerTeamInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::assign_event_player_team_for_user(&state, user_id, event_id, payload)
        .await
        .map(Json)
}

pub async fn set_matchup(
    Path((event_id, match_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SetMatchupInput>,
) -> ApiResult<Match> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::set_matchup_for_user(&state, user_id, event_id, match_id, payload)
        .await
        .map(Json)
}

pub async fn get_event_signup_link(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<EventSignupLinkResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::get_event_signup_link_for_user(&state, user_id, event_id)
        .await
        .map(Json)
}

pub async fn list_event_signup_requests(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Vec<EventSignupRequest>> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::list_signup_requests_for_user(&state, user_id, event_id)
        .await
        .map(Json)
}

pub async fn accept_event_signup_request(
    Path((event_id, request_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::accept_signup_request_for_user(&state, user_id, event_id, request_id)
        .await
        .map(Json)
}

pub async fn decline_event_signup_request(
    Path((event_id, request_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::decline_signup_request_for_user(&state, user_id, event_id, request_id)
        .await
        .map(Json)
}

pub async fn get_public_signup_info(
    Path(signup_token): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<PublicEventSignupInfo> {
    service::get_public_signup_info(&state, &signup_token)
        .await
        .map(Json)
}

pub async fn create_public_signup_request(
    Path(signup_token): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateEventSignupRequestInput>,
) -> ApiResult<MessageResponse> {
    service::create_public_signup_request(&state, &signup_token, payload)
        .await
        .map(Json)
}
