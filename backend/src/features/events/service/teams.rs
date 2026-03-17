use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::models::{AutoBalanceTeamsResponse, CreateEventTeamInput, Event, UpdateEventTeamInput},
        permissions::require_event_owner_access,
    },
    shared::{
        errors::{bad_request, internal_error, not_found, ApiError},
        models::MessageResponse,
    },
};

use super::{
    average_team_elo_from_players, ensure_event_exists, format_team_size,
    pug_role_targets_for_format, rank_elo_for_balance, repo, role_overflow_penalty,
    unique_team_name, BalancePlayer, BalanceTeamState,
};
use super::validation::validate_event_team_name;

pub async fn create_event_team_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: CreateEventTeamInput,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;

    let name = payload.name.trim();
    if name.is_empty() {
        return Err(bad_request("Team name is required"));
    }

    ensure_event_exists(state, event_id).await?;

    let inserted = repo::insert_event_team(&state.pool, event_id, name).await?;

    if !inserted {
        return Err(bad_request("Team name already exists in this event"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}

pub async fn auto_create_solo_teams_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;

    ensure_event_exists(state, event_id).await?;

    let has_matches = repo::event_has_matches(&state.pool, event_id).await?;

    if has_matches {
        return Err(bad_request(
            "Cannot auto-create solo teams after matches already exist",
        ));
    }

    let player_rows = repo::list_unassigned_event_players(&state.pool, event_id).await?;

    if player_rows.is_empty() {
        return Err(bad_request("No unassigned players available"));
    }

    let existing_team_names = repo::list_event_team_names(&state.pool, event_id).await?;

    let mut used_names: HashSet<String> = existing_team_names
        .into_iter()
        .map(|name| name.trim().to_lowercase())
        .collect();

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    for row in player_rows {
        let player_id = row.id;
        let player_name = row.name;

        let base_name = {
            let cleaned = player_name.trim();
            if cleaned.is_empty() {
                "Solo Team".to_string()
            } else {
                cleaned.to_string()
            }
        };

        let team_name = unique_team_name(&base_name, &mut used_names);
        let team_id = Uuid::new_v4();

        repo::insert_event_team_in_tx(&mut tx, event_id, team_id, &team_name).await?;
        repo::insert_event_team_membership_in_tx(&mut tx, event_id, team_id, player_id).await?;
    }

    tx.commit().await.map_err(internal_error)?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}

pub async fn auto_balance_teams_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<AutoBalanceTeamsResponse, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;

    let event = repo::load_event(&state.pool, event_id).await?;
    if event.teams.is_empty() {
        return Err(bad_request("Create at least one team before auto-balancing"));
    }

    let team_size = format_team_size(&event.format);
    if team_size == 0 {
        return Err(bad_request("Invalid event format for auto-balance"));
    }

    let required_players = team_size * event.teams.len();
    if event.players.len() < required_players {
        return Err(bad_request(&format!(
            "Need at least {required_players} players to fill {} teams in {} format",
            event.teams.len(),
            event.format.as_db_value()
        )));
    }

    let role_targets = pug_role_targets_for_format(&event.format);
    let mut ranked_players: Vec<BalancePlayer> = event
        .players
        .iter()
        .map(|player| BalancePlayer {
            id: player.id,
            role: player.role,
            elo: rank_elo_for_balance(player.rank),
        })
        .collect();

    ranked_players.sort_by(|a, b| b.elo.cmp(&a.elo));
    let selected_players: Vec<BalancePlayer> =
        ranked_players.into_iter().take(required_players).collect();

    let total_elo: i32 = selected_players.iter().map(|player| player.elo).sum();
    let target_team_avg = total_elo as f64 / event.teams.len() as f64;

    let mut team_states: Vec<BalanceTeamState> = event
        .teams
        .iter()
        .map(|team| BalanceTeamState::new(team.id, team_size))
        .collect();

    for player in selected_players {
        let mut best_index: Option<usize> = None;
        let mut best_score: Option<f64> = None;

        for (index, team) in team_states.iter().enumerate() {
            if team.player_ids.len() >= team_size {
                continue;
            }

            let projected_elo = team.elo_sum + player.elo;
            let projected_size = team.player_ids.len() + 1;
            let projected_avg = projected_elo as f64 / projected_size as f64;
            let avg_penalty = (projected_avg - target_team_avg).abs();

            let role_penalty = match role_targets {
                Some(targets) => role_overflow_penalty(team, player.role, targets),
                None => 0.0,
            };

            let fill_penalty = projected_size as f64 * 0.01;
            let score = avg_penalty + role_penalty + fill_penalty;

            match best_score {
                Some(current_best) if score >= current_best => {}
                _ => {
                    best_score = Some(score);
                    best_index = Some(index);
                }
            }
        }

        let Some(team_index) = best_index else {
            return Err(bad_request("Unable to build balanced team setup"));
        };

        let target_team = &mut team_states[team_index];
        target_team.add_player(&player);
    }

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    repo::clear_event_team_memberships_in_tx(&mut tx, event_id).await?;

    for team in &team_states {
        for player_id in &team.player_ids {
            repo::insert_event_team_membership_in_tx(&mut tx, event_id, team.id, *player_id)
                .await?;
        }
    }

    tx.commit().await.map_err(internal_error)?;

    let updated_event = repo::load_event(&state.pool, event_id).await?.into_owner(is_owner);

    let mut team_summaries = Vec::new();
    let mut min_avg = f64::MAX;
    let mut max_avg = f64::MIN;
    for team in &updated_event.teams {
        let players: Vec<_> = updated_event
            .players
            .iter()
            .filter(|player| player.team_id == Some(team.id))
            .collect();
        let avg = average_team_elo_from_players(&players);
        if let Some(value) = avg {
            min_avg = min_avg.min(value);
            max_avg = max_avg.max(value);
            team_summaries.push(format!("{}: {}", team.name, value.round() as i32));
        } else {
            team_summaries.push(format!("{}: N/A", team.name));
        }
    }

    let delta = if min_avg.is_finite() && max_avg.is_finite() {
        (max_avg - min_avg).round() as i32
    } else {
        0
    };

    let summary = format!(
        "Balanced {required_players} players across {} teams. Avg ELO delta: {delta}. {}",
        updated_event.teams.len(),
        team_summaries.join(" | ")
    );

    Ok(AutoBalanceTeamsResponse {
        event: updated_event,
        summary,
    })
}

pub async fn delete_event_team_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    team_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let played_matches = repo::count_played_matches_for_team(&state.pool, event_id, team_id).await?;
    if played_matches > 0 {
        return Err(bad_request(
            "Cannot delete a team that already has completed match results.",
        ));
    }

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    let deleted = repo::delete_event_team_by_id_in_tx(&mut tx, event_id, team_id).await?;
    if !deleted {
        return Err(not_found("Team not found in this event"));
    }

    repo::clear_team_from_event_matches_in_tx(&mut tx, event_id, team_id).await?;

    tx.commit().await.map_err(internal_error)?;

    Ok(MessageResponse {
        message: "Team deleted".to_string(),
    })
}

pub async fn update_event_team_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    team_id: Uuid,
    payload: UpdateEventTeamInput,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;
    validate_event_team_name(&payload.name)?;

    let outcome =
        repo::update_event_team_name_by_id(&state.pool, event_id, team_id, payload.name.trim())
            .await?;

    match outcome {
        repo::TeamNameUpdateOutcome::Updated => {}
        repo::TeamNameUpdateOutcome::NotFound => {
            return Err(not_found("Team not found in this event"));
        }
        repo::TeamNameUpdateOutcome::DuplicateName => {
            return Err(bad_request("Team name already exists in this event"));
        }
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}
