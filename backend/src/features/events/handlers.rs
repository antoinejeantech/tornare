use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    shared::{
        errors::ApiResult,
        models::{
            AddPlayerInput, AssignEventPlayerTeamInput, CreateEventInput, CreateEventMatchInput,
            CreateEventTeamInput, Event, Match, MessageResponse, SetMatchupInput, UpdateEventInput,
            UpdateEventPlayerInput, UpdateEventTeamInput,
        },
    },
};

use super::service;

pub async fn list_events(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Vec<Event>> {
    service::list_events_for_headers(&state, &headers).await.map(Json)
}

pub async fn get_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    service::get_event_for_headers(&state, &headers, event_id)
        .await
        .map(Json)
}

pub async fn create_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEventInput>,
) -> ApiResult<Event> {
    service::create_event_for_headers(&state, &headers, payload)
        .await
        .map(Json)
}

pub async fn update_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateEventInput>,
) -> ApiResult<Event> {
    service::update_event_for_headers(&state, &headers, event_id, payload)
        .await
        .map(Json)
}

pub async fn delete_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    service::delete_event_for_headers(&state, &headers, event_id)
        .await
        .map(Json)
}

pub async fn create_event_match(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEventMatchInput>,
) -> ApiResult<Match> {
    service::create_event_match_for_headers(&state, &headers, event_id, payload)
        .await
        .map(Json)
}

pub async fn add_event_player(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AddPlayerInput>,
) -> ApiResult<Event> {
    service::add_event_player_for_headers(&state, &headers, event_id, payload)
        .await
        .map(Json)
}

pub async fn delete_event_player(
    Path((event_id, player_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    service::delete_event_player_for_headers(&state, &headers, event_id, player_id)
        .await
        .map(Json)
}

pub async fn update_event_player(
    Path((event_id, player_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateEventPlayerInput>,
) -> ApiResult<Event> {
    service::update_event_player_for_headers(&state, &headers, event_id, player_id, payload)
        .await
        .map(Json)
}

pub async fn create_event_team(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEventTeamInput>,
) -> ApiResult<Event> {
    service::create_event_team_for_headers(&state, &headers, event_id, payload)
        .await
        .map(Json)
}

pub async fn delete_event_team(
    Path((event_id, team_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    service::delete_event_team_for_headers(&state, &headers, event_id, team_id)
        .await
        .map(Json)
}

pub async fn update_event_team(
    Path((event_id, team_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateEventTeamInput>,
) -> ApiResult<Event> {
    service::update_event_team_for_headers(&state, &headers, event_id, team_id, payload)
        .await
        .map(Json)
}

pub async fn assign_event_player_team(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AssignEventPlayerTeamInput>,
) -> ApiResult<Event> {
    service::assign_event_player_team_for_headers(&state, &headers, event_id, payload)
        .await
        .map(Json)
}

pub async fn set_matchup(
    Path((event_id, match_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SetMatchupInput>,
) -> ApiResult<Match> {
    service::set_matchup_for_headers(&state, &headers, event_id, match_id, payload)
        .await
        .map(Json)
}
