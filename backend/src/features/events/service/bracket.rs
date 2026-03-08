use uuid::Uuid;

use crate::shared::errors::{internal_error, ApiError};

use sqlx::{Row, Transaction};

pub(super) struct BracketMatchPlan {
    pub id: Uuid,
    pub round: i32,
    pub position: i32,
    pub title: String,
    pub map: String,
    pub max_players: u8,
    pub team_a_id: Option<Uuid>,
    pub team_b_id: Option<Uuid>,
    pub next_match_id: Option<Uuid>,
    pub next_match_slot: Option<String>,
    pub winner_team_id: Option<Uuid>,
    pub status: String,
}

pub(super) fn bracket_rounds(bracket_size: usize) -> usize {
    let mut rounds = 0;
    let mut remaining = bracket_size;

    while remaining > 1 {
        remaining /= 2;
        rounds += 1;
    }

    rounds
}

pub(super) async fn normalize_bracket_matches(
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

pub(super) async fn propagate_match_winners(
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
