use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{matches::repo as matches_repo, permissions::require_event_owner_access},
    shared::{
        errors::{bad_request, internal_error, not_found, ApiError},
        models::{
            AddPlayerInput, AssignEventPlayerTeamInput, CreateEventInput, CreateEventMatchInput,
            CreateEventSignupRequestInput, CreateEventTeamInput, CreateMatchInput, Event,
            EventSignupLinkResponse, EventSignupRequest, Match, MessageResponse,
            PublicEventSignupInfo, SetMatchupInput, UpdateEventInput, UpdateEventPlayerInput,
            UpdateEventTeamInput, OVERWATCH_RANKS,
        },
        numeric::{i32_to_u8, i32_to_usize, i64_to_usize},
    },
};

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
        "INSERT INTO events (id, name, description, start_date, event_type, max_players, signup_token)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(event_id)
    .bind(payload.name.trim())
    .bind(payload.description.trim())
    .bind(normalized_start_date)
    .bind(payload.event_type.as_db_value())
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
         SET name = $1, description = $2, start_date = $3, event_type = $4, max_players = $5
         WHERE id = $6
         RETURNING id",
    )
    .bind(payload.name.trim())
    .bind(payload.description.trim())
    .bind(normalized_start_date)
    .bind(payload.event_type.as_db_value())
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

    Ok(())
}

fn validate_update_event_input(payload: &UpdateEventInput) -> Result<(), ApiError> {
    let create_shape = CreateEventInput {
        name: payload.name.clone(),
        description: payload.description.clone(),
        start_date: payload.start_date.clone(),
        event_type: payload.event_type.clone(),
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
