use std::collections::HashSet;

use uuid::Uuid;

mod bracket;
mod team_balance;
mod validation;

use crate::{
    app::state::AppState,
    features::{
        events::models::{
            AddPlayerInput, AssignEventPlayerTeamInput, AutoBalanceTeamsResponse,
            CreateEventInput, CreateEventMatchInput, CreateEventSignupRequestInput,
            CreateEventTeamInput, CreateMatchInput, Event, EventSignupLinkResponse,
            EventSignupRequest, EventType, Match,
            PublicEventSignupInfo, ReportMatchWinnerInput, SetMatchupInput,
            UpdateEventInput, UpdateEventPlayerInput, UpdateEventTeamInput,
        },
        matches::repo as matches_repo,
        permissions::require_event_owner_access,
    },
    shared::{
        errors::{bad_request, internal_error, not_found, ApiError},
        models::MessageResponse,
        numeric::{i32_to_u8, i32_to_usize, i64_to_usize},
    },
};

use sqlx::Row;

use super::repo;
use bracket::{
    bracket_rounds, normalize_bracket_matches, propagate_match_winners, BracketMatchPlan,
};
use team_balance::{
    average_team_elo_from_players, format_team_size, pug_role_targets_for_format,
    rank_elo_for_balance, role_overflow_penalty, unique_team_name, BalancePlayer,
    BalanceTeamState,
};
use validation::{
    normalize_optional_string, validate_add_player_input, validate_create_event_input,
    validate_create_match_input, validate_event_player_update_input, validate_event_team_name,
    validate_signup_request_input, validate_update_event_input,
};

pub async fn list_events_public(
    state: &AppState,
    viewer_user_id: Option<Uuid>,
) -> Result<Vec<Event>, ApiError> {
    let event_ids = repo::list_visible_event_ids(&state.pool).await?;

    let mut events = Vec::with_capacity(event_ids.len());
    for event_id in event_ids {
        let mut event = repo::load_event(&state.pool, event_id).await?;
        event.is_owner = match viewer_user_id {
            Some(user_id) => repo::is_event_owner(&state.pool, event_id, user_id).await?,
            None => false,
        };
        events.push(event);
    }

    Ok(events)
}

pub async fn get_event_public(
    state: &AppState,
    event_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<Event, ApiError> {
    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = match viewer_user_id {
        Some(user_id) => repo::is_event_owner(&state.pool, event_id, user_id).await?,
        None => false,
    };
    Ok(event)
}

pub async fn create_event_for_user(
    state: &AppState,
    user_id: Uuid,
    payload: CreateEventInput,
) -> Result<Event, ApiError> {
    validate_create_event_input(&payload)?;

    let event_id = Uuid::new_v4();
    let signup_token = Uuid::new_v4().to_string();
    let normalized_start_date = normalize_optional_string(&payload.start_date);

    sqlx::query(
           "INSERT INTO events (id, name, description, start_date, event_type, format, max_players, signup_token)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(event_id)
    .bind(payload.name.trim())
    .bind(payload.description.trim())
    .bind(normalized_start_date)
    .bind(payload.event_type.as_db_value())
    .bind(payload.format.as_db_value())
    .bind(i32::from(payload.max_players))
    .bind(signup_token)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO event_memberships (id, event_id, user_id, role) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(user_id)
    .bind("owner")
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn update_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: UpdateEventInput,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;
    validate_update_event_input(&payload)?;
    let normalized_start_date = normalize_optional_string(&payload.start_date);

    let updated = sqlx::query(
        "UPDATE events
            SET name = $1, description = $2, start_date = $3, event_type = $4, format = $5, max_players = $6
            WHERE id = $7
         RETURNING id",
    )
    .bind(payload.name.trim())
    .bind(payload.description.trim())
    .bind(normalized_start_date)
    .bind(payload.event_type.as_db_value())
    .bind(payload.format.as_db_value())
    .bind(i32::from(payload.max_players))
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    if updated.is_none() {
        return Err(not_found("Event not found"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn delete_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let result = sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(event_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found("Event not found"));
    }

    Ok(MessageResponse {
        message: "Event deleted".to_string(),
    })
}

pub async fn create_event_match_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: CreateEventMatchInput,
) -> Result<Match, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    match repo::event_type_for_event(&state.pool, event_id).await? {
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

    match repo::event_type_for_event(&state.pool, event_id).await? {
        Some(EventType::Tourney) => {}
        Some(EventType::Pug) => {
            return Err(bad_request(
                "Bracket generation is only available for TOURNEY events",
            ));
        }
        None => return Err(not_found("Event not found")),
    }

    let team_ids = repo::list_team_ids_for_event(&state.pool, event_id).await?;
    if team_ids.len() < 2 {
        return Err(bad_request(
            "At least 2 teams are required to generate a tournament bracket",
        ));
    }

    let existing_match_count: i64 = sqlx::query("SELECT COUNT(*) AS count FROM event_matches WHERE event_id = $1")
        .bind(event_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?
        .get("count");

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
        if let Some(parent) = plans
            .iter()
            .find(|plan| plan.round as usize == parent_round && plan.position as usize == parent_position)
        {
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
        sqlx::query(
            "INSERT INTO event_matches (
                id, event_id, team_a_id, team_b_id, title, map, max_players,
                round, position, next_match_id, next_match_slot, winner_team_id,
                is_bracket, status
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, $9, $10, $11, $12,
                TRUE, $13
             )",
        )
        .bind(plan.id)
        .bind(event_id)
        .bind(plan.team_a_id)
        .bind(plan.team_b_id)
        .bind(plan.title.as_str())
        .bind(plan.map.as_str())
        .bind(i32::from(plan.max_players))
        .bind(plan.round)
        .bind(plan.position)
        .bind(Option::<Uuid>::None)
        .bind(Option::<&str>::None)
        .bind(plan.winner_team_id)
        .bind(plan.status.as_str())
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    // Write linkage in a second pass so self-referential next_match_id FKs always point to existing rows.
    for plan in &plans {
        sqlx::query(
            "UPDATE event_matches
             SET next_match_id = $1,
                 next_match_slot = $2
             WHERE id = $3 AND event_id = $4",
        )
        .bind(plan.next_match_id)
        .bind(plan.next_match_slot.as_deref())
        .bind(plan.id)
        .bind(event_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    let event = repo::load_event(&state.pool, event_id).await?;
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

    match repo::event_type_for_event(&state.pool, event_id).await? {
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

    let row = sqlx::query(
        "SELECT
            id,
            team_a_id,
            team_b_id,
            winner_team_id,
            is_bracket
         FROM event_matches
         WHERE id = $1 AND event_id = $2",
    )
    .bind(match_id)
    .bind(event_id)
        .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;

    let Some(row) = row else {
        return Err(not_found("Match not found in this event"));
    };

    if !row.get::<bool, _>("is_bracket") {
        return Err(bad_request(
            "Winner reporting is only supported for bracket matches",
        ));
    }

    if row.get::<Option<Uuid>, _>("winner_team_id").is_some() {
        return Err(bad_request("A winner is already set for this match"));
    }

    let team_a_id = row.get::<Option<Uuid>, _>("team_a_id");
    let team_b_id = row.get::<Option<Uuid>, _>("team_b_id");

    let Some(team_a_id) = team_a_id else {
        return Err(bad_request("Matchup is incomplete"));
    };
    let Some(team_b_id) = team_b_id else {
        return Err(bad_request("Matchup is incomplete"));
    };

    if payload.winner_team_id != team_a_id && payload.winner_team_id != team_b_id {
        return Err(bad_request("Winner must be one of the two match teams"));
    }

    sqlx::query("UPDATE event_matches SET winner_team_id = $1, status = 'COMPLETED' WHERE id = $2")
        .bind(payload.winner_team_id)
        .bind(match_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    propagate_match_winners(&mut tx, match_id, payload.winner_team_id).await?;

    tx.commit().await.map_err(internal_error)?;

    matches_repo::load_match(&state.pool, match_id).await
}

pub async fn add_event_player_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: AddPlayerInput,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;
    validate_add_player_input(&payload)?;

    ensure_event_has_capacity_for_new_player(state, event_id).await?;

    sqlx::query(
        "INSERT INTO event_players (id, event_id, name, role, rank) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(payload.name.trim())
    .bind(payload.role.trim())
    .bind(payload.rank.trim())
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn get_event_signup_link_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<EventSignupLinkResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let Some(signup_token) = repo::signup_token_for_event(&state.pool, event_id).await? else {
        return Err(not_found("Event not found"));
    };

    Ok(EventSignupLinkResponse { signup_token })
}

pub async fn rotate_event_signup_link_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<EventSignupLinkResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let new_token = Uuid::new_v4().to_string();
    let updated = repo::rotate_signup_token_for_event(&state.pool, event_id, &new_token).await?;

    if !updated {
        return Err(not_found("Event not found"));
    }

    Ok(EventSignupLinkResponse {
        signup_token: new_token,
    })
}

pub async fn get_public_signup_info(
    state: &AppState,
    signup_token: &str,
) -> Result<PublicEventSignupInfo, ApiError> {
    let Some(info) = repo::event_signup_info_by_token(&state.pool, signup_token).await? else {
        return Err(not_found("Signup link not found"));
    };

    Ok(info)
}

pub async fn create_public_signup_request(
    state: &AppState,
    signup_token: &str,
    payload: CreateEventSignupRequestInput,
) -> Result<MessageResponse, ApiError> {
    validate_signup_request_input(&payload)?;

    let Some(info) = repo::event_signup_info_by_token(&state.pool, signup_token).await? else {
        return Err(not_found("Signup link not found"));
    };

    if info.current_players >= usize::from(info.max_players) {
        return Err(bad_request("Event roster is already full"));
    }

    let clean_name = payload.name.trim();
    if repo::has_pending_signup_request_with_name(&state.pool, info.event_id, clean_name).await? {
        return Err(bad_request(
            "A signup request with this name is already pending",
        ));
    }

    repo::create_signup_request(
        &state.pool,
        info.event_id,
        clean_name,
        payload.role.trim(),
        payload.rank.trim(),
    )
    .await?;

    Ok(MessageResponse {
        message: "Signup request sent".to_string(),
    })
}

pub async fn list_signup_requests_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Vec<EventSignupRequest>, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    ensure_event_exists(state, event_id).await?;

    repo::list_signup_requests_for_event(&state.pool, event_id).await
}

pub async fn accept_signup_request_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    request_id: Uuid,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let Some(request) = repo::get_signup_request(&state.pool, event_id, request_id).await? else {
        return Err(not_found("Signup request not found"));
    };

    if request.status != "pending" {
        return Err(bad_request("This signup request has already been reviewed"));
    }

    ensure_event_has_capacity_for_new_player(state, event_id).await?;

    sqlx::query(
        "INSERT INTO event_players (id, event_id, name, role, rank) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(request.name)
    .bind(request.role)
    .bind(request.rank)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    let updated_count =
        repo::update_signup_request_status(&state.pool, event_id, request_id, "accepted").await?;
    if updated_count == 0 {
        return Err(bad_request("This signup request has already been reviewed"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn decline_signup_request_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    request_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let updated_count =
        repo::update_signup_request_status(&state.pool, event_id, request_id, "declined").await?;
    if updated_count == 0 {
        return Err(not_found("Pending signup request not found"));
    }

    Ok(MessageResponse {
        message: "Signup request declined".to_string(),
    })
}

pub async fn delete_event_player_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    player_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    ensure_event_exists(state, event_id).await?;

    let deleted =
        sqlx::query("DELETE FROM event_players WHERE id = $1 AND event_id = $2 RETURNING id")
            .bind(player_id)
            .bind(event_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?;

    if deleted.is_none() {
        return Err(not_found("Player not found in this event"));
    }

    Ok(MessageResponse {
        message: "Player removed".to_string(),
    })
}

pub async fn update_event_player_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    player_id: Uuid,
    payload: UpdateEventPlayerInput,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;
    validate_event_player_update_input(&payload)?;

    let updated = sqlx::query(
        "UPDATE event_players SET name = $1, role = $2, rank = $3 WHERE id = $4 AND event_id = $5 RETURNING id",
    )
    .bind(payload.name.trim())
    .bind(payload.role.trim())
    .bind(payload.rank.trim())
    .bind(player_id)
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    if updated.is_none() {
        return Err(not_found("Player not found in this event"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn create_event_team_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: CreateEventTeamInput,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let name = payload.name.trim();
    if name.is_empty() {
        return Err(bad_request("Team name is required"));
    }

    ensure_event_exists(state, event_id).await?;

    let inserted = sqlx::query("INSERT INTO event_teams (id, event_id, name) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(name)
        .execute(&state.pool)
        .await;

    if inserted.is_err() {
        return Err(bad_request("Team name already exists in this event"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn auto_create_solo_teams_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    ensure_event_exists(state, event_id).await?;

    let has_matches: bool = sqlx::query("SELECT EXISTS(SELECT 1 FROM event_matches WHERE event_id = $1) AS has_matches")
        .bind(event_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?
        .get("has_matches");

    if has_matches {
        return Err(bad_request(
            "Cannot auto-create solo teams after matches already exist",
        ));
    }

    let player_rows = sqlx::query(
        "SELECT ep.id, ep.name
         FROM event_players ep
         LEFT JOIN event_team_members etm ON etm.event_id = ep.event_id AND etm.event_player_id = ep.id
         WHERE ep.event_id = $1 AND etm.id IS NULL
         ORDER BY ep.name ASC, ep.id ASC",
    )
    .bind(event_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    if player_rows.is_empty() {
        return Err(bad_request("No unassigned players available"));
    }

    let existing_team_names = sqlx::query("SELECT name FROM event_teams WHERE event_id = $1")
        .bind(event_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;

    let mut used_names: HashSet<String> = existing_team_names
        .into_iter()
        .map(|row| row.get::<String, _>("name").trim().to_lowercase())
        .collect();

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    for row in player_rows {
        let player_id: Uuid = row.get("id");
        let player_name: String = row.get("name");

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

        sqlx::query("INSERT INTO event_teams (id, event_id, name) VALUES ($1, $2, $3)")
            .bind(team_id)
            .bind(event_id)
            .bind(team_name)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;

        sqlx::query(
            "INSERT INTO event_team_members (id, event_id, event_team_id, event_player_id)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(team_id)
        .bind(player_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn auto_balance_teams_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<AutoBalanceTeamsResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

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
            role: player.role.clone(),
            elo: rank_elo_for_balance(&player.rank),
        })
        .collect();

    ranked_players.sort_by(|a, b| b.elo.cmp(&a.elo));
    let selected_players: Vec<BalancePlayer> = ranked_players
        .into_iter()
        .take(required_players)
        .collect();

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
                Some(targets) => role_overflow_penalty(team, &player.role, targets),
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

    sqlx::query("DELETE FROM event_team_members WHERE event_id = $1")
        .bind(event_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    for team in &team_states {
        for player_id in &team.player_ids {
            sqlx::query(
                "INSERT INTO event_team_members (id, event_id, event_team_id, event_player_id)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(event_id)
            .bind(team.id)
            .bind(player_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
        }
    }

    tx.commit().await.map_err(internal_error)?;

    let updated_event = as_owner_event(repo::load_event(&state.pool, event_id).await?);

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

    let deleted =
        sqlx::query("DELETE FROM event_teams WHERE id = $1 AND event_id = $2 RETURNING id")
            .bind(team_id)
            .bind(event_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?;

    if deleted.is_none() {
        return Err(not_found("Team not found in this event"));
    }

    sqlx::query("UPDATE event_matches SET team_a_id = NULL WHERE event_id = $1 AND team_a_id = $2")
        .bind(event_id)
        .bind(team_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    sqlx::query("UPDATE event_matches SET team_b_id = NULL WHERE event_id = $1 AND team_b_id = $2")
        .bind(event_id)
        .bind(team_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

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
    require_event_owner_access(state, event_id, user_id).await?;
    validate_event_team_name(&payload.name)?;

    let updated = sqlx::query(
        "UPDATE event_teams SET name = $1 WHERE id = $2 AND event_id = $3 RETURNING id",
    )
    .bind(payload.name.trim())
    .bind(team_id)
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await;

    let updated = match updated {
        Ok(value) => value,
        Err(_) => return Err(bad_request("Team name already exists in this event")),
    };

    if updated.is_none() {
        return Err(not_found("Team not found in this event"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn assign_event_player_team_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: AssignEventPlayerTeamInput,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    if !repo::event_player_exists(&state.pool, event_id, payload.player_id).await? {
        return Err(not_found("Player not found in this event"));
    }

    if let Some(team_id) = payload.team_id {
        if !repo::event_team_exists(&state.pool, event_id, team_id).await? {
            return Err(not_found("Team not found in this event"));
        }

        sqlx::query(
            "INSERT INTO event_team_members (id, event_id, event_team_id, event_player_id)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (event_id, event_player_id)
             DO UPDATE SET event_team_id = EXCLUDED.event_team_id",
        )
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(team_id)
        .bind(payload.player_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    } else {
        sqlx::query("DELETE FROM event_team_members WHERE event_id = $1 AND event_player_id = $2")
            .bind(event_id)
            .bind(payload.player_id)
            .execute(&state.pool)
            .await
            .map_err(internal_error)?;
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn set_matchup_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    match_id: Uuid,
    payload: SetMatchupInput,
) -> Result<Match, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    if !repo::event_match_exists(&state.pool, event_id, match_id).await? {
        return Err(not_found("Match not found in this event"));
    }

    match (payload.team_a_id, payload.team_b_id) {
        (Some(team_a_id), Some(team_b_id)) => {
            if team_a_id == team_b_id {
                return Err(bad_request("A match must have two different teams"));
            }

            if !repo::event_team_exists(&state.pool, event_id, team_a_id).await? {
                return Err(not_found("Team A not found in this event"));
            }
            if !repo::event_team_exists(&state.pool, event_id, team_b_id).await? {
                return Err(not_found("Team B not found in this event"));
            }

            sqlx::query("UPDATE event_matches SET team_a_id = $1, team_b_id = $2 WHERE id = $3")
                .bind(team_a_id)
                .bind(team_b_id)
                .bind(match_id)
                .execute(&state.pool)
                .await
                .map_err(internal_error)?;
        }
        (None, None) => {
            sqlx::query(
                "UPDATE event_matches SET team_a_id = NULL, team_b_id = NULL WHERE id = $1",
            )
            .bind(match_id)
            .execute(&state.pool)
            .await
            .map_err(internal_error)?;
        }
        _ => return Err(bad_request("Provide both teams or clear both")),
    }

    matches_repo::load_match(&state.pool, match_id).await
}

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

async fn create_match_record(
    state: &AppState,
    payload: CreateMatchInput,
    event_id: Uuid,
) -> Result<Match, ApiError> {
    validate_create_match_input(&payload)?;

    let match_id = Uuid::new_v4();

    sqlx::query("INSERT INTO event_matches (id, event_id, title, map, max_players) VALUES ($1, $2, $3, $4, $5)")
        .bind(match_id)
        .bind(event_id)
        .bind(payload.title.trim())
        .bind(payload.map.trim())
        .bind(i32::from(payload.max_players))
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    matches_repo::load_match(&state.pool, match_id).await
}

fn as_owner_event(mut event: Event) -> Event {
    event.is_owner = true;
    event
}
