use uuid::Uuid;

mod events_admin;
mod matches;
mod players;
mod public;
mod signup;
mod teams;
mod team_balance;
mod validation;

use crate::{
    app::state::AppState,
    features::events::models::Event,
    shared::{
        errors::{bad_request, not_found, ApiError},
        numeric::{i32_to_usize, i64_to_usize},
    },
};

use super::repo;
use team_balance::{
    BalancePlayer, BalanceTeamState, average_team_elo_from_players, format_team_size,
    pug_role_targets_for_format, rank_elo_for_balance, role_overflow_penalty, unique_team_name,
};
use validation::{
    normalize_optional_string,
};

pub use signup::{
    accept_signup_request_for_user, create_public_signup_request, decline_signup_request_for_user,
    get_event_signup_link_for_user, get_public_signup_info, list_signup_requests_for_user,
    rotate_event_signup_link_for_user, set_event_public_signup_for_user,
};
pub use players::{
    add_event_player_for_user, assign_event_player_team_for_user, delete_event_player_for_user,
    update_event_player_for_user,
};
pub use matches::{
    cancel_match_winner_for_user, clear_tourney_bracket_for_user, create_event_match_for_user,
    generate_tourney_bracket_for_user, report_match_winner_for_user,
    set_matchup_for_user,
};
pub use public::{get_event_public, list_events_public};
pub use events_admin::{create_event_for_user, delete_event_for_user, update_event_for_user};
pub use teams::{
    auto_balance_teams_for_user, auto_create_solo_teams_for_user, create_event_team_for_user,
    delete_event_team_for_user, update_event_team_for_user,
};

async fn ensure_event_exists(state: &AppState, event_id: Uuid) -> Result<(), ApiError> {
    if !repo::event_exists(&state.pool, event_id).await? {
        return Err(not_found("Event not found"));
    }

    Ok(())
}

async fn event_max_players_i32_or_not_found(
    state: &AppState,
    event_id: Uuid,
) -> Result<i32, ApiError> {
    repo::event_max_players(&state.pool, event_id)
        .await?
        .ok_or_else(|| not_found("Event not found"))
}

async fn ensure_event_has_capacity_for_new_player(
    state: &AppState,
    event_id: Uuid,
) -> Result<(), ApiError> {
    let max_players =
        i32_to_usize(event_max_players_i32_or_not_found(state, event_id).await?, "max_players")?;
    let current_count = i64_to_usize(
        repo::count_event_players(&state.pool, event_id).await?,
        "player count",
    )?;

    if current_count >= max_players {
        return Err(bad_request("Event roster is already full"));
    }

    Ok(())
}

fn as_owner_event(mut event: Event) -> Event {
    event.is_owner = true;
    event
}
