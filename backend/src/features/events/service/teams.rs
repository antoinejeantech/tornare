use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::models::{AutoBalanceTeamsResponse, CreateEventTeamInput, Event, PlayerRole, UpdateEventTeamInput},
        permissions::require_event_owner_access,
    },
    shared::{
        errors::{bad_request, internal_error, not_found, ApiError},
        models::MessageResponse,
    },
};

use super::{
    average_team_elo_from_players, check_role_feasibility, ensure_event_exists, format_team_size,
    find_best_role_balance, pug_role_targets_for_format, rank_elo_for_balance, repo,
    unique_team_name, BalancePlayer, BalanceTeamState, PlayerRoleAssignment, RoleOption,
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
        repo::insert_event_team_membership_in_tx(&mut tx, event_id, team_id, player_id, None, None).await?;
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
    if event.teams.len() != 2 {
        return Err(bad_request("Auto-balance currently supports exactly 2 teams"));
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
        .map(|player| {
            let options: Vec<RoleOption> = if !player.roles.is_empty() {
                player.roles
                    .iter()
                    .map(|pref| RoleOption {
                        role: pref.role,
                        rank: pref.rank,
                        elo: rank_elo_for_balance(pref.rank),
                    })
                    .collect()
            } else {
                vec![RoleOption {
                    role: player.role,
                    rank: player.rank,
                    elo: rank_elo_for_balance(player.rank),
                }]
            };
            BalancePlayer { id: player.id, options }
        })
        .collect();

    ranked_players.sort_by(|a, b| b.max_elo().cmp(&a.max_elo()));

    // Check feasibility against the full pool — gives a meaningful error when
    // the roster genuinely lacks enough role-diverse players.
    if let Some(targets) = role_targets {
        if !check_role_feasibility(&ranked_players, targets, event.teams.len()) {
            return Err(bad_request(&format!(
                "Not enough role diversity to fill {} team(s) in {} format \
                 ({} tank, {} DPS, {} support per team). \
                 Add players with the missing roles and try again.",
                event.teams.len(),
                event.format.as_db_value(),
                targets.tank,
                targets.dps,
                targets.support,
            )));
        }
    }

    // Build the initial balancing pool.
    // For structured formats we do not simply take the top `required_players`
    // by ELO, because that can exclude the lower-ranked flex/role-specialist
    // players needed to make an exact 1/2/2 or 2/2/2 composition possible.
    let selected_players: Vec<BalancePlayer> = match role_targets {
        Some(targets) => {
            let num_teams = event.teams.len();
            let role_needs = [
                (PlayerRole::Tank, targets.tank * num_teams),
                (PlayerRole::Dps, targets.dps * num_teams),
                (PlayerRole::Support, targets.support * num_teams),
            ];
            let mut selected_ids: HashSet<Uuid> = HashSet::new();
            let mut selected: Vec<BalancePlayer> = Vec::with_capacity(required_players);

            // Phase 1: reserve minimum required players per role.
            // Prefer players whose *primary* option matches the role (best fit first),
            // then fall back to any player who can cover it.
            for (role, needed) in role_needs {
                let mut filled = 0usize;
                // Primary-preference pass.
                for player in &ranked_players {
                    if filled >= needed { break; }
                    if selected_ids.contains(&player.id) { continue; }
                    if player.options.first().map(|o| o.role) == Some(role) {
                        selected_ids.insert(player.id);
                        selected.push(player.clone());
                        filled += 1;
                    }
                }
                // Fallback pass for any player who can cover the role.
                for player in &ranked_players {
                    if filled >= needed { break; }
                    if selected_ids.contains(&player.id) { continue; }
                    if player.options.iter().any(|o| o.role == role) {
                        selected_ids.insert(player.id);
                        selected.push(player.clone());
                        filled += 1;
                    }
                }
            }

            // Phase 2: fill remaining spots by ELO.
            for player in &ranked_players {
                if selected.len() >= required_players {
                    break;
                }
                if !selected_ids.contains(&player.id) {
                    selected_ids.insert(player.id);
                    selected.push(player.clone());
                }
            }

            // Re-sort so the later non-role-targeted path still sees the pool in
            // strongest-first order.
            selected.sort_by(|a, b| b.max_elo().cmp(&a.max_elo()));
            selected
        }
        None => ranked_players.iter().take(required_players).cloned().collect(),
    };

    let (team_states, role_assignments): (Vec<BalanceTeamState>, Vec<PlayerRoleAssignment>) =
        match role_targets {
            Some(targets) => {
                let team_ids: Vec<Uuid> = event.teams.iter().map(|team| team.id).collect();
                // Exact role formats use the dedicated solver, which assigns
                // players into explicit role slots and may improve the initial
                // pool by swapping in a few near-cutoff candidates.
                find_best_role_balance(&ranked_players, &selected_players, &team_ids, team_size, targets)
                    .ok_or_else(|| bad_request("Unable to build balanced team setup"))?
            }
            None => {
                // 1v1 has no role-shape constraints, so a simpler greedy ELO
                // balancer is sufficient here.
                let mut team_states: Vec<BalanceTeamState> = event
                    .teams
                    .iter()
                    .map(|team| BalanceTeamState::new(team.id, team_size))
                    .collect();
                let mut role_assignments: Vec<PlayerRoleAssignment> = Vec::new();

                for player in &selected_players {
                    let mut best_team_index: Option<usize> = None;
                    let mut best_opt_index: Option<usize> = None;
                    let mut best_score: Option<f64> = None;

                    for (team_idx, team) in team_states.iter().enumerate() {
                        if team.player_ids.len() >= team_size {
                            continue;
                        }

                        for (opt_idx, option) in player.options.iter().enumerate() {
                            let preference_penalty = opt_idx as f64 * 50.0;
                            let score = (team.elo_sum + option.elo) as f64 + preference_penalty;

                            match best_score {
                                Some(current_best) if score >= current_best => {}
                                _ => {
                                    best_score = Some(score);
                                    best_team_index = Some(team_idx);
                                    best_opt_index = Some(opt_idx);
                                }
                            }
                        }
                    }

                    let (Some(team_index), Some(opt_index)) = (best_team_index, best_opt_index) else {
                        return Err(bad_request("Unable to build balanced team setup"));
                    };

                    let chosen = player.options[opt_index];
                    team_states[team_index].add_player(player.id, chosen.role, chosen.elo);
                    role_assignments.push(PlayerRoleAssignment {
                        player_id: player.id,
                        chosen_role: chosen.role,
                        chosen_rank: chosen.rank,
                    });
                }

                (team_states, role_assignments)
            }
        };

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    repo::clear_event_team_memberships_in_tx(&mut tx, event_id).await?;

    for team in &team_states {
        for player_id in &team.player_ids {
            let a = role_assignments
                .iter()
                .find(|a| a.player_id == *player_id)
                .expect("every balanced player must have a role assignment");
            repo::insert_event_team_membership_in_tx(
                &mut tx,
                event_id,
                team.id,
                *player_id,
                Some(a.chosen_role.as_db_value()),
                Some(a.chosen_rank.as_db_value()),
            )
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
    ensure_event_exists(state, event_id).await?;

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
    ensure_event_exists(state, event_id).await?;
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
