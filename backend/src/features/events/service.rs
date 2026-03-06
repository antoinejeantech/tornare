use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        permissions::{require_event_manage_access, require_event_owner_access, require_event_view_access},
    },
    shared::{
        db::{i32_to_u8, i32_to_usize, i64_to_usize, load_event, load_match},
        errors::{bad_request, internal_error, not_found, ApiError},
        models::{
            AddPlayerInput, AssignEventPlayerTeamInput, CreateEventInput, CreateEventMatchInput,
            CreateEventTeamInput, CreateMatchInput, Event, Match, MessageResponse, SetMatchupInput,
            UpdateEventInput, UpdateEventPlayerInput, UpdateEventTeamInput, OVERWATCH_RANKS,
        },
    },
};

use super::repo;

pub async fn list_events_for_user(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<Event>, ApiError> {
    let event_ids = repo::list_visible_event_ids(&state.pool, user_id).await?;

    let mut events = Vec::with_capacity(event_ids.len());
    for event_id in event_ids {
        events.push(load_event(&state.pool, event_id).await?);
    }

    Ok(events)
}

pub async fn get_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, ApiError> {
    require_event_view_access(state, event_id, user_id).await?;
    load_event(&state.pool, event_id).await
}

pub async fn create_event_for_user(
    state: &AppState,
    user_id: Uuid,
    payload: CreateEventInput,
) -> Result<Event, ApiError> {
    validate_create_event_input(&payload)?;

    let event_id = Uuid::new_v4();

    sqlx::query("INSERT INTO events (id, name, event_type, max_players) VALUES ($1, $2, $3, $4)")
        .bind(event_id)
        .bind(payload.name.trim())
        .bind(payload.event_type.as_db_value())
        .bind(i32::from(payload.max_players))
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    sqlx::query("INSERT INTO event_memberships (id, event_id, user_id, role) VALUES ($1, $2, $3, $4)")
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(user_id)
        .bind("owner")
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    load_event(&state.pool, event_id).await
}

pub async fn update_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: UpdateEventInput,
) -> Result<Event, ApiError> {
    require_event_manage_access(state, event_id, user_id).await?;
    validate_update_event_input(&payload)?;

    let updated = sqlx::query(
        "UPDATE events SET name = $1, event_type = $2, max_players = $3 WHERE id = $4 RETURNING id",
    )
    .bind(payload.name.trim())
    .bind(payload.event_type.as_db_value())
    .bind(i32::from(payload.max_players))
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    if updated.is_none() {
        return Err(not_found("Event not found"));
    }

    load_event(&state.pool, event_id).await
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
    require_event_manage_access(state, event_id, user_id).await?;

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

pub async fn add_event_player_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: AddPlayerInput,
) -> Result<Event, ApiError> {
    require_event_manage_access(state, event_id, user_id).await?;
    validate_add_player_input(&payload)?;

    let Some(max_players_i32) = repo::event_max_players(&state.pool, event_id).await? else {
        return Err(not_found("Event not found"));
    };

    let max_players = i32_to_usize(max_players_i32, "max_players")?;
    let current_count = i64_to_usize(repo::count_event_players(&state.pool, event_id).await?, "player count")?;

    if current_count >= max_players {
        return Err(bad_request("Event roster is already full"));
    }

    sqlx::query("INSERT INTO event_players (id, event_id, name, role, rank) VALUES ($1, $2, $3, $4, $5)")
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(payload.name.trim())
        .bind(payload.role.trim())
        .bind(payload.rank.trim())
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    load_event(&state.pool, event_id).await
}

pub async fn delete_event_player_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    player_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_manage_access(state, event_id, user_id).await?;

    if !repo::event_exists(&state.pool, event_id).await? {
        return Err(not_found("Event not found"));
    }

    let deleted = sqlx::query("DELETE FROM event_players WHERE id = $1 AND event_id = $2 RETURNING id")
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
    require_event_manage_access(state, event_id, user_id).await?;
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

    load_event(&state.pool, event_id).await
}

pub async fn create_event_team_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: CreateEventTeamInput,
) -> Result<Event, ApiError> {
    require_event_manage_access(state, event_id, user_id).await?;

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

    load_event(&state.pool, event_id).await
}

pub async fn delete_event_team_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    team_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_manage_access(state, event_id, user_id).await?;

    if !repo::event_exists(&state.pool, event_id).await? {
        return Err(not_found("Event not found"));
    }

    let deleted = sqlx::query("DELETE FROM event_teams WHERE id = $1 AND event_id = $2 RETURNING id")
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
    require_event_manage_access(state, event_id, user_id).await?;
    validate_event_team_name(&payload.name)?;

    let updated =
        sqlx::query("UPDATE event_teams SET name = $1 WHERE id = $2 AND event_id = $3 RETURNING id")
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

    load_event(&state.pool, event_id).await
}

pub async fn assign_event_player_team_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: AssignEventPlayerTeamInput,
) -> Result<Event, ApiError> {
    require_event_manage_access(state, event_id, user_id).await?;

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

    load_event(&state.pool, event_id).await
}

pub async fn set_matchup_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    match_id: Uuid,
    payload: SetMatchupInput,
) -> Result<Match, ApiError> {
    require_event_manage_access(state, event_id, user_id).await?;

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
            sqlx::query("UPDATE event_matches SET team_a_id = NULL, team_b_id = NULL WHERE id = $1")
                .bind(match_id)
                .execute(&state.pool)
                .await
                .map_err(internal_error)?;
        }
        _ => return Err(bad_request("Provide both teams or clear both")),
    }

    load_match(&state.pool, match_id).await
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

    if !(2..=12).contains(&payload.max_players) {
        return Err(bad_request("Max players must be between 2 and 12"));
    }

    Ok(())
}

fn validate_create_event_input(payload: &CreateEventInput) -> Result<(), ApiError> {
    let name = payload.name.trim();

    if name.is_empty() {
        return Err(bad_request("Event name is required"));
    }

    if !(2..=12).contains(&payload.max_players) {
        return Err(bad_request("Max players must be between 2 and 12"));
    }

    Ok(())
}

fn validate_update_event_input(payload: &UpdateEventInput) -> Result<(), ApiError> {
    let create_shape = CreateEventInput {
        name: payload.name.clone(),
        event_type: payload.event_type.clone(),
        max_players: payload.max_players,
    };

    validate_create_event_input(&create_shape)
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

    load_match(&state.pool, match_id).await
}

fn validate_add_player_input(payload: &AddPlayerInput) -> Result<(), ApiError> {
    let name = payload.name.trim();
    let role = payload.role.trim();
    let rank = payload.rank.trim();

    if name.is_empty() {
        return Err(bad_request("Player name is required"));
    }

    if role.is_empty() {
        return Err(bad_request("Player role is required"));
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
