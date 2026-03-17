use std::collections::HashSet;

use uuid::Uuid;

use crate::features::events::models::{EventFormat, Player, PlayerRank, PlayerRole};

#[derive(Clone)]
pub(super) struct BalancePlayer {
    pub id: Uuid,
    pub role: PlayerRole,
    pub elo: i32,
}

#[derive(Default, Clone, Copy)]
struct RoleCounts {
    tank: usize,
    dps: usize,
    support: usize,
}

impl RoleCounts {
    fn add(&mut self, role: PlayerRole) {
        match role {
            PlayerRole::Tank => self.tank += 1,
            PlayerRole::Dps => self.dps += 1,
            PlayerRole::Support => self.support += 1,
        }
    }

    fn get(&self, role: PlayerRole) -> usize {
        match role {
            PlayerRole::Tank => self.tank,
            PlayerRole::Dps => self.dps,
            PlayerRole::Support => self.support,
        }
    }
}

#[derive(Clone)]
pub(super) struct BalanceTeamState {
    pub id: Uuid,
    pub player_ids: Vec<Uuid>,
    pub elo_sum: i32,
    role_counts: RoleCounts,
}

impl BalanceTeamState {
    pub fn new(id: Uuid, team_size: usize) -> Self {
        Self {
            id,
            player_ids: Vec::with_capacity(team_size),
            elo_sum: 0,
            role_counts: RoleCounts::default(),
        }
    }

    pub fn add_player(&mut self, player: &BalancePlayer) {
        self.player_ids.push(player.id);
        self.elo_sum += player.elo;
        self.role_counts.add(player.role);
    }

    fn role_count(&self, role: PlayerRole) -> usize {
        self.role_counts.get(role)
    }
}

#[derive(Clone, Copy)]
pub(super) struct PugRoleTargets {
    tank: usize,
    dps: usize,
    support: usize,
}

impl PugRoleTargets {
    fn get(&self, role: PlayerRole) -> usize {
        match role {
            PlayerRole::Tank => self.tank,
            PlayerRole::Dps => self.dps,
            PlayerRole::Support => self.support,
        }
    }
}

pub(super) fn format_team_size(format: &EventFormat) -> usize {
    match format {
        EventFormat::OneVOne => 1,
        EventFormat::SixVSix => 6,
        EventFormat::FiveVFive => 5,
    }
}

pub(super) fn pug_role_targets_for_format(format: &EventFormat) -> Option<PugRoleTargets> {
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

pub(super) fn role_overflow_penalty(
    team: &BalanceTeamState,
    role: PlayerRole,
    targets: PugRoleTargets,
) -> f64 {
    let current = team.role_count(role);
    let target = targets.get(role);

    if current + 1 <= target {
        return 0.0;
    }

    ((current + 1 - target) as f64) * 400.0
}

pub(super) fn rank_elo_for_balance(rank: PlayerRank) -> i32 {
    match rank {
        PlayerRank::Bronze => 1000,
        PlayerRank::Silver => 1500,
        PlayerRank::Gold | PlayerRank::Unranked => 2000,
        PlayerRank::Platinum => 2500,
        PlayerRank::Diamond => 3000,
        PlayerRank::Master => 3500,
        PlayerRank::Grandmaster => 4000,
        PlayerRank::Champion => 4500,
    }
}

pub(super) fn average_team_elo_from_players(players: &[&Player]) -> Option<f64> {
    let mut total = 0i32;
    let mut count = 0usize;

    for player in players {
        total += rank_elo_for_balance(player.rank);
        count += 1;
    }

    if count == 0 {
        return None;
    }

    Some(total as f64 / count as f64)
}

pub(super) fn unique_team_name(base_name: &str, used_names: &mut HashSet<String>) -> String {
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
