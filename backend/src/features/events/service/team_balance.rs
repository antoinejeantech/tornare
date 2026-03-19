use std::collections::HashSet;

use uuid::Uuid;

use crate::features::events::models::{EventFormat, Player, PlayerRank, PlayerRole};

// One concrete way a player can be used by the balancer.
// A multi-role player contributes one RoleOption per declared preference.
#[derive(Clone, Copy)]
pub(super) struct RoleOption {
    pub role: PlayerRole,
    pub rank: PlayerRank,
    pub elo: i32,
}

// Compact balance-time view of a roster player.
// `options[0]` is treated as the player's preferred role, and later options are
// valid fallbacks that the search may use if needed to satisfy the team shape.
#[derive(Clone)]
pub(super) struct BalancePlayer {
    pub id: Uuid,
    pub options: Vec<RoleOption>,
}

impl BalancePlayer {
    // Used to sort the pool so the strongest/flex-most valuable players are
    // considered early during roster selection.
    pub fn max_elo(&self) -> i32 {
        self.options.iter().map(|o| o.elo).max().unwrap_or(0)
    }
}

// Final role decision persisted for each player once balancing is done.
#[derive(Clone)]
pub(super) struct PlayerRoleAssignment {
    pub player_id: Uuid,
    pub chosen_role: PlayerRole,
    pub chosen_rank: PlayerRank,
}

#[derive(Default, Clone, Copy)]
struct RoleCounts {
    tank: usize,
    dps: usize,
    support: usize,
}

impl RoleCounts {
    // Internal accounting used while building a team state.
    fn add(&mut self, role: PlayerRole) {
        match role {
            PlayerRole::Tank => self.tank += 1,
            PlayerRole::Dps => self.dps += 1,
            PlayerRole::Support => self.support += 1,
        }
    }
}

// Mutable team snapshot used during search/scoring.
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

    pub fn add_player(&mut self, player_id: Uuid, role: PlayerRole, elo: i32) {
        self.player_ids.push(player_id);
        self.elo_sum += elo;
        self.role_counts.add(role);
    }
}

// Hard per-team composition targets for structured PUG formats.
#[derive(Clone, Copy)]
pub(super) struct PugRoleTargets {
    pub tank: usize,
    pub dps: usize,
    pub support: usize,
}

pub(super) fn format_team_size(format: &EventFormat) -> usize {
    match format {
        EventFormat::OneVOne => 1,
        EventFormat::SixVSix => 6,
        EventFormat::FiveVFive => 5,
    }
}

// 1v1 has no role-shape constraints; 5v5 and 6v6 do.
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

// Fast exact feasibility pre-check.
// This does not choose teams yet; it only answers whether the pool can possibly
// fill all required role slots across all teams.
pub(super) fn check_role_feasibility(
    players: &[BalancePlayer],
    targets: PugRoleTargets,
    num_teams: usize,
) -> bool {
    let need_t = targets.tank * num_teams;
    let need_d = targets.dps * num_teams;
    let need_s = targets.support * num_teams;

    let count = |want_t: bool, want_d: bool, want_s: bool| -> usize {
        players
            .iter()
            .filter(|p| {
                // Count players who can cover at least one role in the subset.
                p.options.iter().any(|o| {
                    (want_t && o.role == PlayerRole::Tank)
                        || (want_d && o.role == PlayerRole::Dps)
                        || (want_s && o.role == PlayerRole::Support)
                })
            })
            .count()
    };

    // Hall-style checks for every role subset.
    // If any subset is short, no assignment exists and the expensive search can stop.
    count(true, false, false) >= need_t
        && count(false, true, false) >= need_d
        && count(false, false, true) >= need_s
        && count(true, true, false) >= need_t + need_d
        && count(true, false, true) >= need_t + need_s
        && count(false, true, true) >= need_d + need_s
        && count(true, true, true) >= need_t + need_d + need_s
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
        total += rank_elo_for_balance(player.assigned_rank.unwrap_or(player.rank));
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

// One required slot in the final output: for example "team 1 needs a Support".
#[derive(Clone, Copy)]
struct RoleSlot {
    team_index: usize,
    role: PlayerRole,
}

// Internal search result used to compare exact assignments.
// Ordering is: smaller ELO spread first, then fewer off-role picks, then more total ELO.
#[derive(Clone)]
struct ExactBalanceResult {
    team_states: Vec<BalanceTeamState>,
    role_assignments: Vec<PlayerRoleAssignment>,
    spread: i32,
    preference_penalty: i32,
    total_elo: i32,
}

// Returns the first option matching the required role plus its preference index.
// The index matters because later preferences are treated as more expensive.
fn option_for_role(player: &BalancePlayer, role: PlayerRole) -> Option<(usize, RoleOption)> {
    player
        .options
        .iter()
        .enumerate()
        .find_map(|(index, option)| (option.role == role).then_some((index, *option)))
}

    // Expand a team format into concrete slots so the search can solve a plain
    // assignment problem instead of reasoning about role counts indirectly.
fn role_slots_for_targets(num_teams: usize, targets: PugRoleTargets) -> Vec<RoleSlot> {
    let mut slots = Vec::with_capacity(num_teams * (targets.tank + targets.dps + targets.support));

    for team_index in 0..num_teams {
        for _ in 0..targets.tank {
            slots.push(RoleSlot {
                team_index,
                role: PlayerRole::Tank,
            });
        }
        for _ in 0..targets.dps {
            slots.push(RoleSlot {
                team_index,
                role: PlayerRole::Dps,
            });
        }
        for _ in 0..targets.support {
            slots.push(RoleSlot {
                team_index,
                role: PlayerRole::Support,
            });
        }
    }

    slots
}

// Lower spread wins; if tied, prefer staying closer to primary roles; if still
// tied, keep the stronger overall lobby on the board.
fn exact_balance_is_better(
    spread: i32,
    preference_penalty: i32,
    total_elo: i32,
    best: &ExactBalanceResult,
) -> bool {
    (spread, preference_penalty, -total_elo) < (best.spread, best.preference_penalty, -best.total_elo)
}

// Depth-first exact search over role slots.
// Each recursive level fills one required slot with one unused player who can
// cover it. This guarantees the final result respects the format exactly.
fn exact_balance_dfs(
    slot_index: usize,
    slots: &[RoleSlot],
    players: &[BalancePlayer],
    used_players: &mut [bool],
    team_elos: &mut [i32],
    assigned_slots: &mut [Option<(usize, RoleOption, usize)>],
    best: &mut Option<ExactBalanceResult>,
    team_ids: &[Uuid],
    team_size: usize,
) {
    if slot_index == slots.len() {
        // All required slots are filled; score the completed assignment.
        let spread = team_elos.iter().max().copied().unwrap_or(0)
            - team_elos.iter().min().copied().unwrap_or(0);
        let preference_penalty = assigned_slots
            .iter()
            .flatten()
            .map(|(_, _, option_index)| *option_index as i32)
            .sum::<i32>();
        let total_elo = team_elos.iter().sum::<i32>();

        let mut team_states: Vec<BalanceTeamState> = team_ids
            .iter()
            .copied()
            .map(|team_id| BalanceTeamState::new(team_id, team_size))
            .collect();
        let mut role_assignments = Vec::with_capacity(players.len());

        for (slot, assignment) in slots.iter().zip(assigned_slots.iter()) {
            let Some((player_index, option, _)) = assignment else {
                return;
            };
            let player = &players[*player_index];
            team_states[slot.team_index].add_player(player.id, option.role, option.elo);
            role_assignments.push(PlayerRoleAssignment {
                player_id: player.id,
                chosen_role: option.role,
                chosen_rank: option.rank,
            });
        }

        let candidate = ExactBalanceResult {
            team_states,
            role_assignments,
            spread,
            preference_penalty,
            total_elo,
        };

        match best {
            Some(current) if !exact_balance_is_better(spread, preference_penalty, total_elo, current) => {}
            _ => *best = Some(candidate),
        }

        return;
    }

    let slot = slots[slot_index];
    let mut candidates = Vec::new();
    for (player_index, player) in players.iter().enumerate() {
        if used_players[player_index] {
            continue;
        }
        if let Some((option_index, option)) = option_for_role(player, slot.role) {
            candidates.push((player_index, option_index, option));
        }
    }

    // Search the most natural candidates first: primary-role fits before flexes,
    // then higher ELO first within the same preference bucket.
    candidates.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| b.2.elo.cmp(&a.2.elo))
            .then_with(|| players[a.0].id.cmp(&players[b.0].id))
    });

    for (player_index, option_index, option) in candidates {
        // Choose this player for the current slot, recurse, then backtrack.
        used_players[player_index] = true;
        team_elos[slot.team_index] += option.elo;
        assigned_slots[slot_index] = Some((player_index, option, option_index));

        exact_balance_dfs(
            slot_index + 1,
            slots,
            players,
            used_players,
            team_elos,
            assigned_slots,
            best,
            team_ids,
            team_size,
        );

        assigned_slots[slot_index] = None;
        team_elos[slot.team_index] -= option.elo;
        used_players[player_index] = false;
    }
}

// Exact search for a fixed set of players.
// Slots are ordered by scarcity so the search hits constrained roles first.
fn find_exact_role_balance(
    players: &[BalancePlayer],
    team_ids: &[Uuid],
    team_size: usize,
    targets: PugRoleTargets,
) -> Option<ExactBalanceResult> {
    if players.is_empty() {
        return None;
    }

    let mut slots = role_slots_for_targets(team_ids.len(), targets);
    slots.sort_by_key(|slot| {
        // Fill rare roles first to prune dead branches earlier.
        let eligible = players
            .iter()
            .filter(|player| player.options.iter().any(|option| option.role == slot.role))
            .count();
        (eligible, slot.team_index)
    });

    let mut used_players = vec![false; players.len()];
    let mut team_elos = vec![0; team_ids.len()];
    let mut assigned_slots = vec![None; slots.len()];
    let mut best = None;

    exact_balance_dfs(
        0,
        &slots,
        players,
        &mut used_players,
        &mut team_elos,
        &mut assigned_slots,
        &mut best,
        team_ids,
        team_size,
    );

    best
}

// Fallback search when the initially selected top-ELO set cannot satisfy the
// exact team shape. This tries combinations from a slightly expanded candidate
// pool instead of blindly trusting the first ELO cutoff.
fn best_subset_dfs(
    start_index: usize,
    candidate_players: &[BalancePlayer],
    required_players: usize,
    chosen_indices: &mut Vec<usize>,
    team_ids: &[Uuid],
    team_size: usize,
    targets: PugRoleTargets,
    best: &mut Option<ExactBalanceResult>,
) {
    if chosen_indices.len() == required_players {
        // Materialize the chosen subset and solve it exactly.
        let chosen_players: Vec<BalancePlayer> = chosen_indices
            .iter()
            .map(|&index| candidate_players[index].clone())
            .collect();
        if !check_role_feasibility(&chosen_players, targets, team_ids.len()) {
            return;
        }
        let Some(candidate) = find_exact_role_balance(&chosen_players, team_ids, team_size, targets) else {
            return;
        };
        match best {
            Some(current)
                if !exact_balance_is_better(
                    candidate.spread,
                    candidate.preference_penalty,
                    candidate.total_elo,
                    current,
                ) => {}
            _ => *best = Some(candidate),
        }
        return;
    }

    let remaining_needed = required_players - chosen_indices.len();
    if candidate_players.len().saturating_sub(start_index) < remaining_needed {
        // Not enough players left to complete a valid subset.
        return;
    }

    for index in start_index..=candidate_players.len() - remaining_needed {
        chosen_indices.push(index);
        best_subset_dfs(
            index + 1,
            candidate_players,
            required_players,
            chosen_indices,
            team_ids,
            team_size,
            targets,
            best,
        );
        chosen_indices.pop();
    }
}

fn try_single_swap_improvements(
    ranked_players: &[BalancePlayer],
    initially_selected_players: &[BalancePlayer],
    team_ids: &[Uuid],
    team_size: usize,
    targets: PugRoleTargets,
    best: &mut Option<ExactBalanceResult>,
) {
    let selected_ids: HashSet<Uuid> = initially_selected_players.iter().map(|player| player.id).collect();
    let extra_candidates: Vec<BalancePlayer> = ranked_players
        .iter()
        .filter(|player| !selected_ids.contains(&player.id))
        .take(6)
        .cloned()
        .collect();

    if extra_candidates.is_empty() {
        return;
    }

    let mut swapped_subset = initially_selected_players.to_vec();
    for extra in &extra_candidates {
        for replace_index in 0..swapped_subset.len() {
            let original = swapped_subset[replace_index].clone();
            swapped_subset[replace_index] = extra.clone();

            if check_role_feasibility(&swapped_subset, targets, team_ids.len()) {
                if let Some(candidate) =
                    find_exact_role_balance(&swapped_subset, team_ids, team_size, targets)
                {
                    match best {
                        Some(current)
                            if !exact_balance_is_better(
                                candidate.spread,
                                candidate.preference_penalty,
                                candidate.total_elo,
                                current,
                            ) => {}
                        _ => *best = Some(candidate),
                    }
                }
            }

            swapped_subset[replace_index] = original;
        }
    }
}

// Public entry point for structured-role balancing.
// Strategy:
// 1. Try the initially selected top-ELO roster exactly.
// 2. If that exact roster cannot satisfy the required role slots, widen the pool
//    a little and search for a better subset that still respects the format.
pub(super) fn find_best_role_balance(
    ranked_players: &[BalancePlayer],
    initially_selected_players: &[BalancePlayer],
    team_ids: &[Uuid],
    team_size: usize,
    targets: PugRoleTargets,
) -> Option<(Vec<BalanceTeamState>, Vec<PlayerRoleAssignment>)> {
    let mut best = find_exact_role_balance(initially_selected_players, team_ids, team_size, targets);

    // Even when the initial top-ELO cutoff is already feasible, it can still be
    // suboptimal. Trying a few one-for-one swaps against the next-best players
    // catches the common case where one excluded flex player creates a clearly
    // better overall balance.
    if best.is_some() {
        try_single_swap_improvements(
            ranked_players,
            initially_selected_players,
            team_ids,
            team_size,
            targets,
            &mut best,
        );
        return best.map(|result| (result.team_states, result.role_assignments));
    }

    let selected_ids: HashSet<Uuid> = initially_selected_players.iter().map(|player| player.id).collect();
    let mut candidate_players = initially_selected_players.to_vec();
    // Keep the fallback bounded so the combinatorial search stays tractable.
    let candidate_limit = initially_selected_players.len() + 6;
    for player in ranked_players {
        if candidate_players.len() >= candidate_limit {
            break;
        }
        if !selected_ids.contains(&player.id) {
            candidate_players.push(player.clone());
        }
    }

    let mut chosen_indices = Vec::with_capacity(initially_selected_players.len());
    let mut best = None;
    best_subset_dfs(
        0,
        &candidate_players,
        initially_selected_players.len(),
        &mut chosen_indices,
        team_ids,
        team_size,
        targets,
        &mut best,
    );

    best.map(|result| (result.team_states, result.role_assignments))
}
