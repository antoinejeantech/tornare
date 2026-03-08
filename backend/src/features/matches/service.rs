use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::{
            models::{
                CreateEventMatchInput, CreateMatchInput, Event, EventType, Match,
                ReportMatchWinnerInput, SetMatchupInput,
            },
            repo as events_repo,
        },
        permissions::{require_event_manage_access, require_event_view_access},
        permissions::require_event_owner_access,
    },
    shared::{
        errors::{bad_request, internal_error, not_found, ApiError},
        models::MessageResponse,
        numeric::i32_to_u8,
    },
};

use sqlx::Transaction;

use super::repo;

pub async fn list_matches_for_user(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<Match>, ApiError> {
    let match_ids = repo::list_visible_match_ids(&state.pool, user_id).await?;

    let mut matches = Vec::with_capacity(match_ids.len());
    for match_id in match_ids {
        matches.push(repo::load_match(&state.pool, match_id).await?);
    }

    Ok(matches)
}

pub async fn get_match_for_user(
    state: &AppState,
    user_id: Uuid,
    match_id: Uuid,
) -> Result<Match, ApiError> {
    let Some(event_id) = repo::get_match_event_id(&state.pool, match_id).await? else {
        return Err(not_found("Match not found"));
    };

    require_event_view_access(state, event_id, user_id).await?;
    repo::load_match(&state.pool, match_id).await
}

pub async fn delete_match_for_user(
    state: &AppState,
    user_id: Uuid,
    match_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    let Some(event_id) = repo::get_match_event_id(&state.pool, match_id).await? else {
        return Err(not_found("Match not found"));
    };

    require_event_manage_access(state, event_id, user_id).await?;

    let deleted = repo::delete_match_by_id(&state.pool, match_id).await?;
    if deleted == 0 {
        return Err(not_found("Match not found"));
    }

    Ok(MessageResponse {
        message: "Match deleted".to_string(),
    })
}

pub async fn create_event_match_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: CreateEventMatchInput,
) -> Result<Match, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    match events_repo::event_type_for_event(&state.pool, event_id).await? {
        Some(EventType::Tourney) => {
            return Err(bad_request(
                "Tournament matches are generated from the bracket. Manual creation is disabled.",
            ));
        }
        Some(EventType::Pug) => {}
        None => return Err(not_found("Event not found")),
    }

    let max_players_i32 = event_max_players_i32_or_not_found(state, event_id).await?;

    let create_match = CreateMatchInput {
        title: payload.title,
        map: payload.map,
        max_players: i32_to_u8(max_players_i32, "max_players")?,
    };

    create_match_record(state, create_match, event_id).await
}

pub async fn generate_tourney_bracket_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    match events_repo::event_type_for_event(&state.pool, event_id).await? {
        Some(EventType::Tourney) => {}
        Some(EventType::Pug) => {
            return Err(bad_request(
                "Bracket generation is only available for TOURNEY events",
            ));
        }
        None => return Err(not_found("Event not found")),
    }

    let team_ids = events_repo::list_team_ids_for_event(&state.pool, event_id).await?;
    if team_ids.len() < 2 {
        return Err(bad_request(
            "At least 2 teams are required to generate a tournament bracket",
        ));
    }

    let existing_match_count = repo::count_event_matches(&state.pool, event_id).await?;

    if existing_match_count > 0 {
        return Err(bad_request(
            "This event already has matches. Clear them before generating a bracket.",
        ));
    }

    let max_players_i32 = event_max_players_i32_or_not_found(state, event_id).await?;
    let max_players = i32_to_u8(max_players_i32, "max_players")?;

    let bracket_size = team_ids.len().next_power_of_two();
    let rounds = bracket_rounds(bracket_size);

    let mut plans: Vec<BracketMatchPlan> = Vec::new();
    for round in 1..=rounds {
        let matches_in_round = bracket_size >> round;
        for position in 1..=matches_in_round {
            plans.push(BracketMatchPlan {
                id: Uuid::new_v4(),
                round: round as i32,
                position: position as i32,
                title: format!("Round {round} Match {position}"),
                map: "TBD".to_string(),
                max_players,
                team_a_id: None,
                team_b_id: None,
                next_match_id: None,
                next_match_slot: None,
                winner_team_id: None,
                status: "OPEN".to_string(),
            });
        }
    }

    for idx in 0..plans.len() {
        let round = plans[idx].round as usize;
        let position = plans[idx].position as usize;
        if round >= rounds {
            continue;
        }

        let parent_round = round + 1;
        let parent_position = (position + 1) / 2;
        if let Some(parent) = plans.iter().find(|plan| {
            plan.round as usize == parent_round && plan.position as usize == parent_position
        }) {
            plans[idx].next_match_id = Some(parent.id);
            plans[idx].next_match_slot = Some(if position % 2 == 1 {
                "A".to_string()
            } else {
                "B".to_string()
            });
        }
    }

    let mut seeded: Vec<Option<Uuid>> = team_ids.into_iter().map(Some).collect();
    while seeded.len() < bracket_size {
        seeded.push(None);
    }

    for plan in plans.iter_mut().filter(|plan| plan.round == 1) {
        let position = (plan.position as usize) - 1;
        plan.team_a_id = seeded.get(position * 2).copied().flatten();
        plan.team_b_id = seeded.get(position * 2 + 1).copied().flatten();
    }

    let mut changed = true;
    while changed {
        changed = false;

        for idx in 0..plans.len() {
            let (team_a_id, team_b_id) = (plans[idx].team_a_id, plans[idx].team_b_id);
            if plans[idx].winner_team_id.is_none() {
                match (team_a_id, team_b_id) {
                    (Some(team_id), None) | (None, Some(team_id)) => {
                        plans[idx].winner_team_id = Some(team_id);
                        plans[idx].status = "COMPLETED".to_string();
                        changed = true;
                    }
                    (Some(_), Some(_)) => {
                        if plans[idx].status != "READY" {
                            plans[idx].status = "READY".to_string();
                            changed = true;
                        }
                    }
                    (None, None) => {
                        if plans[idx].status != "OPEN" {
                            plans[idx].status = "OPEN".to_string();
                            changed = true;
                        }
                    }
                }
            }

            if let Some(winner) = plans[idx].winner_team_id {
                let Some(next_match_id) = plans[idx].next_match_id else {
                    continue;
                };
                let next_slot = plans[idx].next_match_slot.clone();

                if let Some(next_idx) = plans.iter().position(|plan| plan.id == next_match_id) {
                    match next_slot.as_deref() {
                        Some("A") => {
                            if plans[next_idx].team_a_id != Some(winner) {
                                plans[next_idx].team_a_id = Some(winner);
                                changed = true;
                            }
                        }
                        Some("B") => {
                            if plans[next_idx].team_b_id != Some(winner) {
                                plans[next_idx].team_b_id = Some(winner);
                                changed = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    for plan in &plans {
        repo::insert_bracket_match_in_tx(
            &mut tx,
            repo::BracketMatchInsert {
                id: plan.id,
                event_id,
                team_a_id: plan.team_a_id,
                team_b_id: plan.team_b_id,
                title: plan.title.as_str(),
                map: plan.map.as_str(),
                max_players: i32::from(plan.max_players),
                round: plan.round,
                position: plan.position,
                winner_team_id: plan.winner_team_id,
                status: plan.status.as_str(),
            },
        )
        .await?;
    }

    for plan in &plans {
        repo::update_bracket_next_link_in_tx(
            &mut tx,
            plan.id,
            event_id,
            plan.next_match_id,
            plan.next_match_slot.as_deref(),
        )
        .await?;
    }

    tx.commit().await.map_err(internal_error)?;

    let event = events_repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn report_match_winner_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    match_id: Uuid,
    payload: ReportMatchWinnerInput,
) -> Result<Match, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    match events_repo::event_type_for_event(&state.pool, event_id).await? {
        Some(EventType::Tourney) => {}
        Some(EventType::Pug) => {
            return Err(bad_request(
                "Winner reporting through bracket progression is only available for TOURNEY events",
            ));
        }
        None => return Err(not_found("Event not found")),
    }

    let mut tx = state.pool.begin().await.map_err(internal_error)?;
    normalize_bracket_matches(&mut tx, event_id).await?;

    let Some(match_state) = repo::get_bracket_match_state_in_tx(&mut tx, event_id, match_id).await? else {
        return Err(not_found("Match not found in this event"));
    };

    if !match_state.is_bracket {
        return Err(bad_request(
            "Winner reporting is only supported for bracket matches",
        ));
    }

    if match_state.winner_team_id.is_some() {
        return Err(bad_request("A winner is already set for this match"));
    }

    let team_a_id = match_state.team_a_id;
    let team_b_id = match_state.team_b_id;

    let Some(team_a_id) = team_a_id else {
        return Err(bad_request("Matchup is incomplete"));
    };
    let Some(team_b_id) = team_b_id else {
        return Err(bad_request("Matchup is incomplete"));
    };

    if payload.winner_team_id != team_a_id && payload.winner_team_id != team_b_id {
        return Err(bad_request("Winner must be one of the two match teams"));
    }

    repo::set_match_winner_completed_in_tx(&mut tx, match_id, payload.winner_team_id).await?;

    propagate_match_winners(&mut tx, match_id, payload.winner_team_id).await?;

    tx.commit().await.map_err(internal_error)?;

    repo::load_match(&state.pool, match_id).await
}

pub async fn set_matchup_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    match_id: Uuid,
    payload: SetMatchupInput,
) -> Result<Match, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    if !events_repo::event_match_exists(&state.pool, event_id, match_id).await? {
        return Err(not_found("Match not found in this event"));
    }

    match (payload.team_a_id, payload.team_b_id) {
        (Some(team_a_id), Some(team_b_id)) => {
            if team_a_id == team_b_id {
                return Err(bad_request("A match must have two different teams"));
            }

            if !events_repo::event_team_exists(&state.pool, event_id, team_a_id).await? {
                return Err(not_found("Team A not found in this event"));
            }
            if !events_repo::event_team_exists(&state.pool, event_id, team_b_id).await? {
                return Err(not_found("Team B not found in this event"));
            }

            repo::set_matchup(&state.pool, match_id, team_a_id, team_b_id).await?;
        }
        (None, None) => {
            repo::clear_matchup(&state.pool, match_id).await?;
        }
        _ => return Err(bad_request("Provide both teams or clear both")),
    }

    repo::load_match(&state.pool, match_id).await
}

fn as_owner_event(mut event: Event) -> Event {
    event.is_owner = true;
    event
}

async fn event_max_players_i32_or_not_found(
    state: &AppState,
    event_id: Uuid,
) -> Result<i32, ApiError> {
    events_repo::event_max_players(&state.pool, event_id)
        .await?
        .ok_or_else(|| not_found("Event not found"))
}

async fn create_match_record(
    state: &AppState,
    payload: CreateMatchInput,
    event_id: Uuid,
) -> Result<Match, ApiError> {
    validate_create_match_input(&payload)?;

    let match_id = Uuid::new_v4();

    repo::insert_event_match(
        &state.pool,
        match_id,
        event_id,
        payload.title.trim(),
        payload.map.trim(),
        i32::from(payload.max_players),
    )
    .await?;

    repo::load_match(&state.pool, match_id).await
}

fn validate_create_match_input(payload: &CreateMatchInput) -> Result<(), ApiError> {
    let title = payload.title.trim();
    let map = payload.map.trim();

    if title.is_empty() {
        return Err(bad_request("Match title is required"));
    }

    if map.is_empty() {
        return Err(bad_request("Map is required"));
    }

    if !(2..=99).contains(&payload.max_players) {
        return Err(bad_request("Max players must be between 2 and 99"));
    }

    Ok(())
}

struct BracketMatchPlan {
    id: Uuid,
    round: i32,
    position: i32,
    title: String,
    map: String,
    max_players: u8,
    team_a_id: Option<Uuid>,
    team_b_id: Option<Uuid>,
    next_match_id: Option<Uuid>,
    next_match_slot: Option<String>,
    winner_team_id: Option<Uuid>,
    status: String,
}

fn bracket_rounds(bracket_size: usize) -> usize {
    let mut rounds = 0;
    let mut remaining = bracket_size;

    while remaining > 1 {
        remaining /= 2;
        rounds += 1;
    }

    rounds
}

async fn normalize_bracket_matches(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
) -> Result<(), ApiError> {
    repo::normalize_bracket_matches_in_tx(tx, event_id).await
}

async fn propagate_match_winners(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    source_match_id: Uuid,
    winner_team_id: Uuid,
) -> Result<(), ApiError> {
    let mut queue: Vec<(Uuid, Uuid)> = vec![(source_match_id, winner_team_id)];

    while let Some((current_match_id, current_winner_team_id)) = queue.pop() {
        let Some((next_match_id, next_match_slot)) =
            repo::get_next_match_link_in_tx(tx, current_match_id).await?
        else {
            continue;
        };

        match next_match_slot.as_deref() {
            Some("A") => {
                repo::set_matchup_slot_in_tx(tx, next_match_id, "A", current_winner_team_id)
                    .await?;
            }
            Some("B") => {
                repo::set_matchup_slot_in_tx(tx, next_match_id, "B", current_winner_team_id)
                    .await?;
            }
            _ => continue,
        }

        let (team_a_id, team_b_id, winner_already_set) =
            repo::get_match_state_in_tx(tx, next_match_id).await?;

        if winner_already_set.is_some() {
            continue;
        }

        match (team_a_id, team_b_id) {
            (Some(_), Some(_)) => {
                repo::set_match_status_in_tx(tx, next_match_id, "READY").await?;
            }
            (Some(_), None) | (None, Some(_)) => {
                repo::set_match_status_in_tx(tx, next_match_id, "OPEN").await?;
            }
            (None, None) => {
                repo::set_match_status_in_tx(tx, next_match_id, "OPEN").await?;
            }
        }
    }

    Ok(())
}
