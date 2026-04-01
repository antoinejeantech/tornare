use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    app::{security::enforce_rate_limit, state::AppState},
    features::{
        auth::{maybe_authenticated_user_id, require_authenticated_user_id, strict_maybe_authenticated_user_id},
        events::models::{
            AddPlayerInput, AssignEventPlayerTeamInput,
            AutoBalanceTeamsResponse, CreateEventInput, CreateEventMatchInput,
            CreateEventSignupRequestInput, CreateEventTeamInput, Event, EventSignupLinkResponse,
            EventSignupRequest, EventsKpiResponse, GenerateTourneyBracketInput, ListEventsQuery,
            Match, PaginatedEventsResponse, PublicEventSignupInfo, ReportMatchWinnerInput,
            SetEventFeaturedInput, SetEventPublicSignupInput, SetMatchupInput, UpdateEventInput,
            UpdateEventPlayerInput, UpdateEventTeamInput, UpdateMatchStartDateInput,
        },
    },
    shared::{errors::ApiResult, models::MessageResponse},
};

use super::service;

pub async fn list_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListEventsQuery>,
) -> ApiResult<PaginatedEventsResponse> {
    let viewer_user_id = maybe_authenticated_user_id(&state, &headers);
    service::list_events_public(&state, viewer_user_id, query)
        .await
        .map(Json)
}

pub async fn get_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    let viewer_user_id = strict_maybe_authenticated_user_id(&state, &headers)?;
    service::get_event_public(&state, event_id, viewer_user_id)
        .await
        .map(Json)
}

pub async fn get_featured_event(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Option<Event>> {
    let viewer_user_id = strict_maybe_authenticated_user_id(&state, &headers)?;
    service::get_featured_event_public(&state, viewer_user_id)
        .await
        .map(Json)
}

pub async fn get_events_kpis(
    State(state): State<AppState>,
) -> ApiResult<EventsKpiResponse> {
    service::get_events_kpis_public(&state)
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

pub async fn auto_create_solo_teams(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::auto_create_solo_teams_for_user(&state, user_id, event_id)
        .await
        .map(Json)
}

pub async fn auto_balance_teams(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<AutoBalanceTeamsResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::auto_balance_teams_for_user(&state, user_id, event_id)
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

pub async fn generate_tourney_bracket(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    payload: Option<Json<GenerateTourneyBracketInput>>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    let mode = payload
        .map(|Json(input)| input.mode)
        .unwrap_or(crate::features::events::models::BracketGenerationMode::Random);
    service::generate_tourney_bracket_for_user(&state, user_id, event_id, mode)
        .await
        .map(Json)
}

pub async fn clear_tourney_bracket(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::clear_tourney_bracket_for_user(&state, user_id, event_id)
        .await
        .map(Json)
}

pub async fn report_match_winner(
    Path((event_id, match_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReportMatchWinnerInput>,
) -> ApiResult<Match> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::report_match_winner_for_user(&state, user_id, event_id, match_id, payload)
        .await
        .map(Json)
}

pub async fn cancel_match_winner(
    Path((event_id, match_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Match> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::cancel_match_winner_for_user(&state, user_id, event_id, match_id)
        .await
        .map(Json)
}

pub async fn update_match_start_date(
    Path((event_id, match_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateMatchStartDateInput>,
) -> ApiResult<Match> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::update_match_start_date_for_user(&state, user_id, event_id, match_id, payload)
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

pub async fn rotate_event_signup_link(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<EventSignupLinkResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::rotate_event_signup_link_for_user(&state, user_id, event_id)
        .await
        .map(Json)
}

pub async fn set_event_public_signup(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SetEventPublicSignupInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::set_event_public_signup_for_user(&state, user_id, event_id, payload.enabled)
        .await
        .map(Json)
}

pub async fn set_event_featured(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SetEventFeaturedInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::set_featured_event_for_user(&state, user_id, event_id, payload.featured)
        .await
        .map(Json)
}

pub async fn publish_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::publish_event_for_user(&state, user_id, event_id)
        .await
        .map(Json)
}

pub async fn unpublish_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::unpublish_event_for_user(&state, user_id, event_id)
        .await
        .map(Json)
}

pub async fn end_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::end_event_for_user(&state, user_id, event_id)
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
    headers: HeaderMap,
    Json(payload): Json<CreateEventSignupRequestInput>,
) -> ApiResult<MessageResponse> {
    enforce_rate_limit(
        &state.rate_limiter,
        &headers,
        "public_event_signup_request",
        8,
        60,
    )
    .await?;

    service::create_public_signup_request(&state, &signup_token, payload, &headers)
        .await
        .map(Json)
}
