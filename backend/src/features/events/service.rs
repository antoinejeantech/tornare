use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::models::{
            AddPlayerInput, AssignEventPlayerTeamInput, AutoBalanceTeamsResponse,
            CreateEventInput, CreateEventMatchInput, CreateEventSignupRequestInput,
            CreateEventTeamInput, CreateMatchInput, Event, EventFormat,
            EventSignupLinkResponse, EventSignupRequest, EventType, Match, Player,
            PublicEventSignupInfo, ReportMatchWinnerInput, SetMatchupInput,
            UpdateEventInput, UpdateEventPlayerInput, UpdateEventTeamInput,
        },
        matches::repo as matches_repo,
        permissions::require_event_owner_access,
        users::models::OVERWATCH_RANKS,
    },
    shared::{
        errors::{bad_request, internal_error, not_found, ApiError},
        models::MessageResponse,
        numeric::{i32_to_u8, i32_to_usize, i64_to_usize},
    },
};

use sqlx::{Row, Transaction};

use super::repo;

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

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
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

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
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

    let Some(max_players_i32) = repo::event_max_players(&state.pool, event_id).await? else {
        return Err(not_found("Event not found"));
    };

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

    let Some(max_players_i32) = repo::event_max_players(&state.pool, event_id).await? else {
        return Err(not_found("Event not found"));
    };
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

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
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

async fn normalize_bracket_matches(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
) -> Result<(), ApiError> {
    // Historical auto-advance bug: some bracket matches were marked completed with only one side filled.
    sqlx::query(
        "UPDATE event_matches
         SET winner_team_id = NULL
         WHERE event_id = $1
           AND is_bracket = TRUE
           AND winner_team_id IS NOT NULL
           AND (team_a_id IS NULL OR team_b_id IS NULL)",
    )
    .bind(event_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    // Keep status consistent with matchup/winner state.
    sqlx::query(
        "UPDATE event_matches
         SET status = CASE
             WHEN winner_team_id IS NOT NULL THEN 'COMPLETED'
             WHEN team_a_id IS NOT NULL AND team_b_id IS NOT NULL THEN 'READY'
             ELSE 'OPEN'
         END
         WHERE event_id = $1
           AND is_bracket = TRUE",
    )
    .bind(event_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn add_event_player_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: AddPlayerInput,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;
    validate_add_player_input(&payload)?;

    let Some(max_players_i32) = repo::event_max_players(&state.pool, event_id).await? else {
        return Err(not_found("Event not found"));
    };

    let max_players = i32_to_usize(max_players_i32, "max_players")?;
    let current_count = i64_to_usize(
        repo::count_event_players(&state.pool, event_id).await?,
        "player count",
    )?;

    if current_count >= max_players {
        return Err(bad_request("Event roster is already full"));
    }

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

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
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

    if !repo::event_exists(&state.pool, event_id).await? {
        return Err(not_found("Event not found"));
    }

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

    let Some(max_players_i32) = repo::event_max_players(&state.pool, event_id).await? else {
        return Err(not_found("Event not found"));
    };

    let max_players = i32_to_usize(max_players_i32, "max_players")?;
    let current_count = i64_to_usize(
        repo::count_event_players(&state.pool, event_id).await?,
        "player count",
    )?;
    if current_count >= max_players {
        return Err(bad_request("Event roster is already full"));
    }

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

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
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

    if !repo::event_exists(&state.pool, event_id).await? {
        return Err(not_found("Event not found"));
    }

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

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
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

    if !repo::event_exists(&state.pool, event_id).await? {
        return Err(not_found("Event not found"));
    }

    let inserted = sqlx::query("INSERT INTO event_teams (id, event_id, name) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(name)
        .execute(&state.pool)
        .await;

    if inserted.is_err() {
        return Err(bad_request("Team name already exists in this event"));
    }

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
}

pub async fn auto_create_solo_teams_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    if !repo::event_exists(&state.pool, event_id).await? {
        return Err(not_found("Event not found"));
    }

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

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
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
        .map(|team| BalanceTeamState {
            id: team.id,
            player_ids: Vec::with_capacity(team_size),
            elo_sum: 0,
            role_counts: RoleCounts::default(),
        })
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
        target_team.player_ids.push(player.id);
        target_team.elo_sum += player.elo;
        target_team.role_counts.add(&player.role);
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

    let mut updated_event = repo::load_event(&state.pool, event_id).await?;
    updated_event.is_owner = true;

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

    if !repo::event_exists(&state.pool, event_id).await? {
        return Err(not_found("Event not found"));
    }

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

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
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

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = true;
    Ok(event)
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

async fn propagate_match_winners(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    source_match_id: Uuid,
    winner_team_id: Uuid,
) -> Result<(), ApiError> {
    let mut queue: Vec<(Uuid, Uuid)> = vec![(source_match_id, winner_team_id)];

    while let Some((current_match_id, current_winner_team_id)) = queue.pop() {
        let row = sqlx::query(
            "SELECT next_match_id, next_match_slot
             FROM event_matches
             WHERE id = $1",
        )
        .bind(current_match_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(internal_error)?;

        let Some(row) = row else {
            continue;
        };

        let next_match_id: Option<Uuid> = row.get("next_match_id");
        let next_match_slot: Option<String> = row.get("next_match_slot");

        let Some(next_match_id) = next_match_id else {
            continue;
        };

        match next_match_slot.as_deref() {
            Some("A") => {
                sqlx::query("UPDATE event_matches SET team_a_id = $1 WHERE id = $2")
                    .bind(current_winner_team_id)
                    .bind(next_match_id)
                    .execute(&mut **tx)
                    .await
                    .map_err(internal_error)?;
            }
            Some("B") => {
                sqlx::query("UPDATE event_matches SET team_b_id = $1 WHERE id = $2")
                    .bind(current_winner_team_id)
                    .bind(next_match_id)
                    .execute(&mut **tx)
                    .await
                    .map_err(internal_error)?;
            }
            _ => continue,
        }

        let next_row = sqlx::query(
            "SELECT team_a_id, team_b_id, winner_team_id
             FROM event_matches
             WHERE id = $1",
        )
        .bind(next_match_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(internal_error)?;

        let team_a_id: Option<Uuid> = next_row.get("team_a_id");
        let team_b_id: Option<Uuid> = next_row.get("team_b_id");
        let winner_already_set: Option<Uuid> = next_row.get("winner_team_id");

        if winner_already_set.is_some() {
            continue;
        }

        match (team_a_id, team_b_id) {
            (Some(_), Some(_)) => {
                sqlx::query("UPDATE event_matches SET status = 'READY' WHERE id = $1")
                    .bind(next_match_id)
                    .execute(&mut **tx)
                    .await
                    .map_err(internal_error)?;
            }
            (Some(_), None) | (None, Some(_)) => {
                // Keep waiting for the missing side; do not auto-advance winners.
                sqlx::query("UPDATE event_matches SET status = 'OPEN' WHERE id = $1")
                    .bind(next_match_id)
                    .execute(&mut **tx)
                    .await
                    .map_err(internal_error)?;
            }
            (None, None) => {
                sqlx::query("UPDATE event_matches SET status = 'OPEN' WHERE id = $1")
                    .bind(next_match_id)
                    .execute(&mut **tx)
                    .await
                    .map_err(internal_error)?;
            }
        }
    }

    Ok(())
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

#[derive(Clone)]
struct BalancePlayer {
    id: Uuid,
    role: String,
    elo: i32,
}

#[derive(Default, Clone, Copy)]
struct RoleCounts {
    tank: usize,
    dps: usize,
    support: usize,
}

impl RoleCounts {
    fn add(&mut self, role: &str) {
        match role {
            "Tank" => self.tank += 1,
            "DPS" => self.dps += 1,
            "Support" => self.support += 1,
            _ => {}
        }
    }

    fn get(&self, role: &str) -> usize {
        match role {
            "Tank" => self.tank,
            "DPS" => self.dps,
            "Support" => self.support,
            _ => 0,
        }
    }
}

#[derive(Clone)]
struct BalanceTeamState {
    id: Uuid,
    player_ids: Vec<Uuid>,
    elo_sum: i32,
    role_counts: RoleCounts,
}

#[derive(Clone, Copy)]
struct PugRoleTargets {
    tank: usize,
    dps: usize,
    support: usize,
}

impl PugRoleTargets {
    fn get(&self, role: &str) -> usize {
        match role {
            "Tank" => self.tank,
            "DPS" => self.dps,
            "Support" => self.support,
            _ => usize::MAX,
        }
    }
}

fn format_team_size(format: &EventFormat) -> usize {
    match format {
        EventFormat::OneVOne => 1,
        EventFormat::SixVSix => 6,
        EventFormat::FiveVFive => 5,
    }
}

fn pug_role_targets_for_format(format: &EventFormat) -> Option<PugRoleTargets> {
    match format {
        EventFormat::FiveVFive => Some(PugRoleTargets {
            tank: 1,
            dps: 2,
            support: 2,
        }),
        EventFormat::SixVSix => Some(PugRoleTargets {
            tank: 2,
            dps: 2,
            support: 2,
        }),
        EventFormat::OneVOne => None,
    }
}

fn role_overflow_penalty(team: &BalanceTeamState, role: &str, targets: PugRoleTargets) -> f64 {
    let current = team.role_counts.get(role);
    let target = targets.get(role);
    if target == usize::MAX {
        return 500.0;
    }

    if current + 1 <= target {
        return 0.0;
    }

    ((current + 1 - target) as f64) * 400.0
}

fn rank_elo_for_balance(rank: &str) -> i32 {
    match rank {
        "Bronze" => 1000,
        "Silver" => 1500,
        "Gold" => 2000,
        "Platinum" => 2500,
        "Diamond" => 3000,
        "Master" => 3500,
        "Grandmaster" => 4000,
        "Champion" => 4500,
        // Frontend exposes Unranked as null ELO; use Gold midpoint for balancing.
        _ => 2000,
    }
}

fn average_team_elo_from_players(players: &[&Player]) -> Option<f64> {
    let mut total = 0i32;
    let mut count = 0usize;

    for player in players {
        total += rank_elo_for_balance(&player.rank);
        count += 1;
    }

    if count == 0 {
        return None;
    }

    Some(total as f64 / count as f64)
}

fn unique_team_name(base_name: &str, used_names: &mut HashSet<String>) -> String {
    let normalized_base = if base_name.trim().is_empty() {
        "Solo Team"
    } else {
        base_name.trim()
    };

    if used_names.insert(normalized_base.to_lowercase()) {
        return normalized_base.to_string();
    }

    let mut suffix = 2usize;
    loop {
        let candidate = format!("{} ({suffix})", normalized_base);
        if used_names.insert(candidate.to_lowercase()) {
            return candidate;
        }
        suffix += 1;
    }
}

fn validate_create_event_input(payload: &CreateEventInput) -> Result<(), ApiError> {
    let name = payload.name.trim();
    let description = payload.description.trim();

    if name.is_empty() {
        return Err(bad_request("Event name is required"));
    }

    if name.len() > 120 {
        return Err(bad_request("Event name must be 120 characters or fewer"));
    }

    if description.len() > 5000 {
        return Err(bad_request(
            "Event description must be 5000 characters or fewer",
        ));
    }

    if let Some(start_date) = normalize_optional_string(&payload.start_date) {
        if start_date.len() > 40 {
            return Err(bad_request("Event start date is too long"));
        }
    }

    if !(2..=99).contains(&payload.max_players) {
        return Err(bad_request("Max players must be between 2 and 99"));
    }

    match &payload.event_type {
        EventType::Pug => {
            if !matches!(
                &payload.format,
                EventFormat::FiveVFive | EventFormat::SixVSix
            ) {
                return Err(bad_request("PUG events support only 5v5 or 6v6 format"));
            }
        }
        EventType::Tourney => {}
    }

    Ok(())
}

fn validate_update_event_input(payload: &UpdateEventInput) -> Result<(), ApiError> {
    let create_shape = CreateEventInput {
        name: payload.name.clone(),
        description: payload.description.clone(),
        start_date: payload.start_date.clone(),
        event_type: payload.event_type.clone(),
        format: payload.format.clone(),
        max_players: payload.max_players,
    };

    validate_create_event_input(&create_shape)
}

fn normalize_optional_string(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
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

fn validate_add_player_input(payload: &AddPlayerInput) -> Result<(), ApiError> {
    let name = payload.name.trim();
    let role = payload.role.trim();
    let rank = payload.rank.trim();

    if name.is_empty() {
        return Err(bad_request("Player name is required"));
    }

    if name.len() > 60 {
        return Err(bad_request("Player name must be 60 characters or fewer"));
    }

    if role.is_empty() {
        return Err(bad_request("Player role is required"));
    }

    if !matches!(role, "Tank" | "DPS" | "Support") {
        return Err(bad_request("Role must be Tank, DPS, or Support"));
    }

    if rank.is_empty() {
        return Err(bad_request("Player rank is required"));
    }

    if !OVERWATCH_RANKS.contains(&rank) {
        return Err(bad_request("Invalid player rank"));
    }

    Ok(())
}

fn validate_event_player_update_input(payload: &UpdateEventPlayerInput) -> Result<(), ApiError> {
    let add_player_shape = AddPlayerInput {
        name: payload.name.clone(),
        role: payload.role.clone(),
        rank: payload.rank.clone(),
    };

    validate_add_player_input(&add_player_shape)
}

fn validate_event_team_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() {
        return Err(bad_request("Team name is required"));
    }

    Ok(())
}

fn validate_signup_request_input(payload: &CreateEventSignupRequestInput) -> Result<(), ApiError> {
    let add_player_shape = AddPlayerInput {
        name: payload.name.clone(),
        role: payload.role.clone(),
        rank: payload.rank.clone(),
    };

    validate_add_player_input(&add_player_shape)
}
