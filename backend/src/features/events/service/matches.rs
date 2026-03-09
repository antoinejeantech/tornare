use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::models::{
            BracketGenerationMode, CreateEventMatchInput, Event, Match,
            ReportMatchWinnerInput, SetMatchupInput,
        },
        matches::service as matches_service,
    },
    shared::errors::ApiError,
};

pub async fn create_event_match_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: CreateEventMatchInput,
) -> Result<Match, ApiError> {
    matches_service::create_event_match_for_user(state, user_id, event_id, payload).await
}

pub async fn generate_tourney_bracket_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    mode: BracketGenerationMode,
) -> Result<Event, ApiError> {
    matches_service::generate_tourney_bracket_for_user(state, user_id, event_id, mode).await
}

pub async fn clear_tourney_bracket_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, ApiError> {
    matches_service::clear_tourney_bracket_for_user(state, user_id, event_id).await
}

pub async fn report_match_winner_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    match_id: Uuid,
    payload: ReportMatchWinnerInput,
) -> Result<Match, ApiError> {
    matches_service::report_match_winner_for_user(state, user_id, event_id, match_id, payload)
        .await
}

pub async fn cancel_match_winner_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    match_id: Uuid,
) -> Result<Match, ApiError> {
    matches_service::cancel_match_winner_for_user(state, user_id, event_id, match_id).await
}

pub async fn set_matchup_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    match_id: Uuid,
    payload: SetMatchupInput,
) -> Result<Match, ApiError> {
    matches_service::set_matchup_for_user(state, user_id, event_id, match_id, payload).await
}
