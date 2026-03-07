<script setup>
import { computed, inject, reactive, ref } from 'vue'
import { getRankElo } from '../../lib/ranks'

const ctx = inject('eventCtx')
const assignmentSearchByTeam = reactive({})
const quickAssignTeamByPlayer = reactive({})
const quickAssignSearch = ref('')

const effectivePugFormat = computed(() => {
  const format = String(ctx.event?.format || '').trim().toLowerCase()
  if (format === '6v6') {
    return '6v6'
  }

  return '5v5'
})

const pugRoleTargets = computed(() => {
  if (effectivePugFormat.value === '6v6') {
    return { Tank: 2, DPS: 2, Support: 2 }
  }

  return { Tank: 1, DPS: 2, Support: 2 }
})

const pugTeamSize = computed(() => {
  const targets = pugRoleTargets.value
  return targets.Tank + targets.DPS + targets.Support
})

const rosterRoleCounts = computed(() => {
  if (!ctx.event) {
    return { Tank: 0, DPS: 0, Support: 0 }
  }

  const counts = { Tank: 0, DPS: 0, Support: 0 }
  for (const player of ctx.event.players) {
    if (player.role === 'Tank' || player.role === 'DPS' || player.role === 'Support') {
      counts[player.role] += 1
    }
  }

  return counts
})

const maxBalancedTeamsFromRoster = computed(() => {
  if (!ctx.event) {
    return 0
  }

  const targets = pugRoleTargets.value
  const roles = rosterRoleCounts.value
  const byRole = Math.min(
    Math.floor(roles.Tank / targets.Tank),
    Math.floor(roles.DPS / targets.DPS),
    Math.floor(roles.Support / targets.Support)
  )

  const byTotal = Math.floor(ctx.event.players.length / pugTeamSize.value)
  return Math.max(0, Math.min(byRole, byTotal))
})

function normalizeSearch(value) {
  return String(value || '')
    .toLowerCase()
    .replace(/\s+/g, ' ')
    .trim()
}

function searchTokens(value) {
  const normalized = normalizeSearch(value)
  return normalized ? normalized.split(' ') : []
}

function playerSearchBlob(player) {
  return normalizeSearch(`${player?.name || ''} ${player?.role || ''} ${player?.rank || ''} ${player?.team || ''}`)
}

function playerMatchesTokens(player, tokens) {
  if (tokens.length === 0) {
    return true
  }

  const blob = playerSearchBlob(player)
  return tokens.every((token) => blob.includes(token))
}

const unassignedPlayers = computed(() => {
  if (!ctx.event) {
    return []
  }

  return ctx.event.players
    .filter((player) => !player.team_id)
    .sort((a, b) => a.name.localeCompare(b.name))
})

const unassignedPlayersCount = computed(() => {
  return unassignedPlayers.value.length
})

const filteredUnassignedPlayers = computed(() => {
  const tokens = searchTokens(quickAssignSearch.value)
  if (tokens.length === 0) {
    return unassignedPlayers.value
  }

  return unassignedPlayers.value.filter((player) => {
    return playerMatchesTokens(player, tokens)
  })
})

function playersForTeam(teamId) {
  if (!ctx.event) {
    return []
  }

  const rolePriority = {
    Tank: 0,
    DPS: 1,
    Support: 2,
  }

  return ctx.event.players
    .filter((player) => player.team_id === teamId)
    .sort((a, b) => {
      const aPriority = rolePriority[a.role] ?? 99
      const bPriority = rolePriority[b.role] ?? 99
      if (aPriority !== bPriority) {
        return aPriority - bPriority
      }

      return a.name.localeCompare(b.name)
    })
}

function teamRoleCounts(teamId) {
  const counts = { Tank: 0, DPS: 0, Support: 0 }
  for (const player of playersForTeam(teamId)) {
    if (player.role === 'Tank' || player.role === 'DPS' || player.role === 'Support') {
      counts[player.role] += 1
    }
  }

  return counts
}

function roleStatusClass(teamId, role) {
  const count = teamRoleCounts(teamId)[role]
  const target = pugRoleTargets.value[role]
  if (count < target) {
    return 'missing'
  }
  if (count > target) {
    return 'excess'
  }

  return 'ok'
}

function teamBalanceNeeds(teamId) {
  const counts = teamRoleCounts(teamId)
  const targets = pugRoleTargets.value
  const needs = []

  for (const role of ['Tank', 'DPS', 'Support']) {
    const missing = targets[role] - counts[role]
    if (missing > 0) {
      needs.push(`${role} x${missing}`)
    }
  }

  return needs.join(', ')
}

function teamBalanceExcess(teamId) {
  const counts = teamRoleCounts(teamId)
  const targets = pugRoleTargets.value
  const extra = []

  for (const role of ['Tank', 'DPS', 'Support']) {
    const overflow = counts[role] - targets[role]
    if (overflow > 0) {
      extra.push(`${role} x${overflow}`)
    }
  }

  return extra.join(', ')
}

function playersAssignableToTeam(teamId) {
  if (!ctx.event) {
    return []
  }

  return ctx.event.players
    .filter((player) => player.team_id !== teamId)
    .sort((a, b) => {
      const aUnassigned = !a.team_id
      const bUnassigned = !b.team_id
      if (aUnassigned !== bUnassigned) {
        return aUnassigned ? -1 : 1
      }

      return a.name.localeCompare(b.name)
    })
}

function assignmentSearchTerm(teamId) {
  return normalizeSearch(assignmentSearchByTeam[teamId])
}

function assignmentSearchValue(teamId) {
  return String(assignmentSearchByTeam[teamId] || '')
}

function setAssignmentSearch(teamId, value) {
  assignmentSearchByTeam[teamId] = String(value || '')
}

function quickAssignSelectedTeamId(playerId) {
  return String(quickAssignTeamByPlayer[playerId] || '')
}

function quickAssignTargetTeam(playerId) {
  const selectedTeamId = quickAssignSelectedTeamId(playerId)
  if (!selectedTeamId || !ctx.event) {
    return null
  }

  return ctx.event.teams.find((team) => team.id === selectedTeamId) || null
}

function quickAssignDisabled(playerId) {
  const targetTeam = quickAssignTargetTeam(playerId)
  if (!targetTeam) {
    return true
  }

  return Boolean(ctx.savingPlayerTeams?.[playerId])
}

function quickAssignBusy(playerId) {
  return Boolean(ctx.savingPlayerTeams?.[playerId])
}

async function quickAssignPlayer(playerId) {
  const targetTeam = quickAssignTargetTeam(playerId)
  if (!targetTeam) {
    return
  }

  await ctx.assignPlayerToTeam(playerId, targetTeam.id)
  quickAssignTeamByPlayer[playerId] = ''
}

function filteredPlayersAssignableToTeam(teamId) {
  const players = playersAssignableToTeam(teamId)
  const tokens = searchTokens(assignmentSearchTerm(teamId))
  if (tokens.length === 0) {
    return []
  }

  return players.filter((player) => {
    return playerMatchesTokens(player, tokens)
  })
}

function visibleTeamAssignResults(teamId) {
  return filteredPlayersAssignableToTeam(teamId).slice(0, 10)
}

function hasTeamAssignmentSearch(teamId) {
  return searchTokens(assignmentSearchTerm(teamId)).length > 0
}

function selectedAssignablePlayer(teamId) {
  const selectedId = String(ctx.teamAssignmentSelections?.[teamId] || '')
  if (!selectedId) {
    return null
  }

  return playersAssignableToTeam(teamId).find((player) => player.id === selectedId) || null
}

function selectedAssignDisabled(teamId) {
  const selectedPlayer = selectedAssignablePlayer(teamId)
  if (!selectedPlayer) {
    return true
  }

  return Boolean(ctx.savingPlayerTeams?.[selectedPlayer.id])
}

function selectedAssignBusy(teamId) {
  const selectedPlayer = selectedAssignablePlayer(teamId)
  if (!selectedPlayer) {
    return false
  }

  return Boolean(ctx.savingPlayerTeams?.[selectedPlayer.id])
}

function roleIcon(role) {
  if (role === 'Tank') {
    return 'shield'
  }
  if (role === 'Support') {
    return 'medical_services'
  }
  return 'swords'
}

function startEditTeam(team) {
  ctx.editingTeamId = team.id
  ctx.editTeamName = team.name
}

function cancelEditTeam() {
  ctx.editingTeamId = null
  ctx.editTeamName = ''
}

function averageEloForTeam(teamId) {
  const eloValues = playersForTeam(teamId)
    .map((player) => getRankElo(player.rank))
    .filter((value) => typeof value === 'number')

  if (eloValues.length === 0) {
    return null
  }

  const total = eloValues.reduce((sum, value) => sum + value, 0)
  return Math.round(total / eloValues.length)
}

function formatTeamAverageElo(teamId) {
  const avg = averageEloForTeam(teamId)
  if (avg === null) {
    return 'Avg ELO: N/A'
  }

  return `Avg ELO: ${avg.toLocaleString()}`
}

function assignmentNotice(player) {
  if (!player?.team_id || !player?.team) {
    return ''
  }

  return `Currently in ${player.team}`
}
</script>

<template>
  <section>
    <h3 class="section-title">
      <span class="material-symbols-rounded section-title-icon" aria-hidden="true">shield</span>
      <span>Teams</span>
    </h3>
    <form v-if="ctx.canManageEvent" class="grid-form compact-form" @submit.prevent="ctx.createTeam">
      <label>
        Team name
        <input v-model="ctx.newTeamName" placeholder="Team Alpha" />
      </label>
      <button type="submit" class="btn-primary" :disabled="!ctx.canCreateTeam || ctx.creatingTeam">
        {{ ctx.creatingTeam ? 'Creating...' : 'Create team' }}
      </button>
    </form>
    <div v-if="ctx.canManageEvent" class="solo-team-action-row">
      <button
        class="btn-secondary"
        :disabled="ctx.creatingSoloTeams || unassignedPlayersCount === 0"
        @click="ctx.autoCreateSoloTeams"
      >
        {{ ctx.creatingSoloTeams ? 'Creating solo teams...' : `Auto-create solo teams (${unassignedPlayersCount})` }}
      </button>
      <button
        class="btn-secondary"
        :disabled="ctx.balancingTeams || ctx.event.teams.length === 0"
        @click="ctx.autoBalanceTeams"
      >
        {{ ctx.balancingTeams ? 'Balancing teams...' : 'Best team setup (ELO)' }}
      </button>
      <p class="muted">Creates one team per unassigned player.</p>
    </div>

    <div v-if="ctx.canManageEvent && ctx.lastBalanceSummary" class="balance-report-box" role="status" aria-live="polite">
      <p class="balance-report-title">Last auto-balance report</p>
      <p class="balance-report-text">{{ ctx.lastBalanceSummary }}</p>
    </div>

    <div v-if="ctx.canManageEvent && !ctx.isTourneyEvent && ctx.event.teams.length > 0" class="balance-helper-panel">
      <div class="balance-helper-head">
        <p class="balance-helper-title">PUG balance assistant</p>
        <span class="balance-helper-format-label">Format: {{ effectivePugFormat }}</span>
      </div>
      <p class="muted balance-helper-summary">
        Current roster can support up to {{ maxBalancedTeamsFromRoster }} fully balanced teams for {{ effectivePugFormat }}.
      </p>
      <div class="balance-roster-row">
        <span class="balance-roster-chip">Tank: {{ rosterRoleCounts.Tank }}</span>
        <span class="balance-roster-chip">DPS: {{ rosterRoleCounts.DPS }}</span>
        <span class="balance-roster-chip">Support: {{ rosterRoleCounts.Support }}</span>
      </div>
    </div>

    <div v-if="ctx.canManageEvent && unassignedPlayersCount > 0" class="quick-assign-panel">
      <div class="quick-assign-header">
        <p class="quick-assign-title">Unassigned players</p>
        <p class="muted">{{ unassignedPlayersCount }} total</p>
      </div>

      <p v-if="ctx.event.teams.length === 0" class="muted">Create at least one team to start assigning players.</p>

      <div v-else class="quick-assign-body">
        <input
          v-model="quickAssignSearch"
          type="search"
          placeholder="Search unassigned by name, role, rank..."
        />
        <p class="muted quick-assign-count">{{ filteredUnassignedPlayers.length }} shown</p>

        <p v-if="filteredUnassignedPlayers.length === 0" class="muted">No unassigned players match this search.</p>

        <ul v-else class="quick-assign-list">
          <li v-for="player in filteredUnassignedPlayers" :key="`quick-assign-${player.id}`" class="quick-assign-item">
            <span class="quick-assign-player">{{ player.name }} · {{ player.role }} · {{ player.rank }}</span>
            <div class="quick-assign-controls">
              <label class="sr-only" :for="`quick-assign-team-${player.id}`">Assign team for {{ player.name }}</label>
              <select :id="`quick-assign-team-${player.id}`" v-model="quickAssignTeamByPlayer[player.id]">
                <option value="">Select team</option>
                <option v-for="team in ctx.event.teams" :key="`quick-team-option-${player.id}-${team.id}`" :value="team.id">
                  {{ team.name }}
                </option>
              </select>
              <button class="btn-secondary" :disabled="quickAssignDisabled(player.id)" @click="quickAssignPlayer(player.id)">
                {{ quickAssignBusy(player.id) ? 'Assigning...' : 'Assign' }}
              </button>
            </div>
          </li>
        </ul>
      </div>
    </div>

    <p v-if="ctx.event.teams.length === 0" class="muted">No teams yet. Create teams first.</p>
    <ul v-else class="entry-list">
      <li v-for="team in ctx.event.teams" :key="team.id" class="team-row">
        <div class="list-main">
          <div v-if="ctx.editingTeamId === team.id" class="inline-edit-row">
            <input v-model="ctx.editTeamName" placeholder="Team name" />
            <button
              class="btn-primary icon-btn"
              :disabled="Boolean(ctx.savingTeamEdits[team.id])"
              :title="ctx.savingTeamEdits[team.id] ? 'Saving team' : 'Save team'"
              @click="ctx.saveTeamEdit(team.id)"
            >
              <span class="material-symbols-rounded" aria-hidden="true">
                {{ ctx.savingTeamEdits[team.id] ? 'hourglass_top' : 'save' }}
              </span>
              <span class="sr-only">{{ ctx.savingTeamEdits[team.id] ? 'Saving team' : 'Save team' }}</span>
            </button>
            <button class="btn-secondary icon-btn" title="Cancel editing team" @click="cancelEditTeam">
              <span class="material-symbols-rounded" aria-hidden="true">close</span>
              <span class="sr-only">Cancel editing team</span>
            </button>
          </div>
          <span v-else class="entry-title">{{ team.name }}</span>
          <div class="team-meta-row muted">
            <span>{{ team.player_ids.length }} players</span>
            <span>{{ formatTeamAverageElo(team.id) }}</span>
          </div>
          <div v-if="ctx.canManageEvent && !ctx.isTourneyEvent" class="team-balance-row">
            <span class="team-balance-pill" :class="roleStatusClass(team.id, 'Tank')">Tank {{ teamRoleCounts(team.id).Tank }}/{{ pugRoleTargets.Tank }}</span>
            <span class="team-balance-pill" :class="roleStatusClass(team.id, 'DPS')">DPS {{ teamRoleCounts(team.id).DPS }}/{{ pugRoleTargets.DPS }}</span>
            <span class="team-balance-pill" :class="roleStatusClass(team.id, 'Support')">Support {{ teamRoleCounts(team.id).Support }}/{{ pugRoleTargets.Support }}</span>
          </div>
          <p v-if="ctx.canManageEvent && !ctx.isTourneyEvent && teamBalanceNeeds(team.id)" class="muted team-balance-note">Needs: {{ teamBalanceNeeds(team.id) }}</p>
          <p v-if="ctx.canManageEvent && !ctx.isTourneyEvent && teamBalanceExcess(team.id)" class="muted team-balance-note">Over target: {{ teamBalanceExcess(team.id) }}</p>
          <ul v-if="playersForTeam(team.id).length > 0" class="team-player-list">
            <li v-for="player in playersForTeam(team.id)" :key="player.id" class="team-player-item">
              <span class="team-player-main">
                <span class="team-player-name">{{ player.name }}</span>
                <img
                  class="team-player-rank-icon"
                  :src="ctx.getRankIcon(player.rank)"
                  :alt="`${player.rank} rank`"
                  :title="`${player.name} · ${player.rank}`"
                />
                <span class="team-player-role">
                  <span class="material-symbols-rounded team-role-icon" aria-hidden="true">{{ roleIcon(player.role) }}</span>
                  <span>{{ player.role }}</span>
                </span>
              </span>
              <button
                v-if="ctx.canManageEvent"
                class="btn-secondary icon-btn team-player-remove"
                :disabled="Boolean(ctx.savingPlayerTeams[player.id])"
                :title="ctx.savingPlayerTeams[player.id] ? 'Removing from team' : 'Remove from team'"
                @click="ctx.removePlayerFromTeam(player.id)"
              >
                <span class="material-symbols-rounded" aria-hidden="true">
                  {{ ctx.savingPlayerTeams[player.id] ? 'hourglass_top' : 'link_off' }}
                </span>
                <span class="sr-only">{{ ctx.savingPlayerTeams[player.id] ? 'Removing from team' : 'Remove from team' }}</span>
              </button>
            </li>
          </ul>
          <span v-else class="muted team-player-empty">No players assigned</span>
          <div v-if="ctx.canManageEvent" class="team-assign-grid">
            <p v-if="playersAssignableToTeam(team.id).length === 0" class="muted team-player-empty">No available players to assign</p>
            <div v-else class="team-assign-row">
              <label class="sr-only" :for="`assign-search-${team.id}`">Search assignable players for {{ team.name }}</label>
              <input
                :id="`assign-search-${team.id}`"
                :value="assignmentSearchValue(team.id)"
                type="search"
                placeholder="Search player, role, rank..."
                @input="setAssignmentSearch(team.id, $event.target.value)"
              />
              <p class="muted team-assign-match-count">{{ filteredPlayersAssignableToTeam(team.id).length }} matches</p>

              <div
                v-if="hasTeamAssignmentSearch(team.id)"
                class="team-assign-dropdown"
                role="listbox"
                :aria-label="`Search results for ${team.name}`"
              >
                <p v-if="filteredPlayersAssignableToTeam(team.id).length === 0" class="muted team-player-empty">No players match this search.</p>

                <ul v-else class="team-assign-results">
                  <li v-for="player in visibleTeamAssignResults(team.id)" :key="`assign-result-${team.id}-${player.id}`">
                    <button
                      class="btn-secondary team-assign-result-btn"
                      :disabled="Boolean(ctx.savingPlayerTeams[player.id])"
                      @click="ctx.assignPlayerToTeam(player.id, team.id)"
                    >
                      <span class="material-symbols-rounded" aria-hidden="true">
                        {{ ctx.savingPlayerTeams[player.id] ? 'hourglass_top' : 'person_add' }}
                      </span>
                      <span class="team-assign-main">{{ player.name }} · {{ player.role }} · {{ player.rank }}</span>
                      <span v-if="assignmentNotice(player)" class="team-assign-notice">{{ assignmentNotice(player) }}</span>
                    </button>
                  </li>
                </ul>

                <p v-if="filteredPlayersAssignableToTeam(team.id).length > 10" class="muted team-assign-limit-note">
                  Showing first 10 matches. Refine search to narrow results.
                </p>
              </div>
            </div>
          </div>
        </div>
        <div class="team-actions">
          <button
            v-if="ctx.canManageEvent && ctx.editingTeamId !== team.id"
            class="btn-secondary icon-btn"
            title="Edit team"
            @click="startEditTeam(team)"
          >
            <span class="material-symbols-rounded" aria-hidden="true">edit</span>
            <span class="sr-only">Edit team</span>
          </button>
          <button
            v-if="ctx.canManageEvent && ctx.editingTeamId !== team.id"
            class="btn-danger icon-btn"
            :disabled="Boolean(ctx.deletingTeams[team.id])"
            :title="ctx.deletingTeams[team.id] ? 'Deleting team' : 'Delete team'"
            @click="ctx.deleteTeam(team)"
          >
            <span class="material-symbols-rounded" aria-hidden="true">
              {{ ctx.deletingTeams[team.id] ? 'hourglass_top' : 'delete' }}
            </span>
            <span class="sr-only">{{ ctx.deletingTeams[team.id] ? 'Deleting team' : 'Delete team' }}</span>
          </button>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.section-title {
  margin: 0 0 0.3rem;
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
}

.section-title-icon {
  font-size: 1.12rem;
  line-height: 1;
}

.grid-form {
  display: grid;
  gap: 0.56rem;
  margin-bottom: 0.72rem;
}

.grid-form label {
  display: grid;
  gap: 0.28rem;
}

.solo-team-action-row {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  flex-wrap: wrap;
  margin: -0.2rem 0 0.72rem;
}

.solo-team-action-row .muted {
  margin: 0;
}

.balance-report-box {
  border: 1px solid color-mix(in srgb, var(--line) 88%, var(--brand-2) 12%);
  background: color-mix(in srgb, var(--card) 94%, #eef6ff 6%);
  border-radius: 10px;
  padding: 0.52rem 0.6rem;
  margin: -0.18rem 0 0.72rem;
}

.balance-report-title {
  margin: 0;
  font-weight: 760;
  color: var(--ink-1);
}

.balance-report-text {
  margin: 0.2rem 0 0;
  color: var(--ink-2);
  font-size: 0.9rem;
}

.balance-helper-panel {
  border: 1px solid color-mix(in srgb, var(--line) 88%, var(--brand-1) 12%);
  background: color-mix(in srgb, var(--card) 93%, #edf4ff 7%);
  border-radius: 10px;
  padding: 0.56rem;
  margin: -0.18rem 0 0.72rem;
  display: grid;
  gap: 0.46rem;
}

.balance-helper-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.6rem;
  flex-wrap: wrap;
}

.balance-helper-title {
  margin: 0;
  font-weight: 760;
}

.balance-helper-format-label {
  font-size: 0.86rem;
  font-weight: 700;
  color: var(--ink-2);
}

.balance-helper-summary {
  margin: 0;
}

.balance-roster-row {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.38rem;
}

.balance-roster-chip {
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-2) 16%);
  background: color-mix(in srgb, var(--card) 92%, #f2f7ff 8%);
  border-radius: 999px;
  padding: 0.16rem 0.5rem;
  font-size: 0.82rem;
  font-weight: 700;
}

.quick-assign-panel {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-1) 10%);
  background: color-mix(in srgb, var(--card) 92%, #eef4ff 8%);
  border-radius: 10px;
  padding: 0.6rem;
  margin: -0.18rem 0 0.72rem;
  display: grid;
  gap: 0.45rem;
}

.quick-assign-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.quick-assign-title {
  margin: 0;
  font-weight: 760;
}

.quick-assign-body {
  display: grid;
  gap: 0.45rem;
}

.quick-assign-count {
  margin: 0;
}

.quick-assign-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.35rem;
  max-height: 16rem;
  overflow: auto;
}

.quick-assign-item {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-2) 8%);
  background: color-mix(in srgb, var(--card) 94%, #f2f7ff 6%);
  border-radius: 8px;
  padding: 0.42rem 0.5rem;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.45rem;
  align-items: center;
}

.quick-assign-player {
  min-width: 0;
  font-weight: 700;
  color: var(--ink-1);
}

.quick-assign-controls {
  display: grid;
  grid-template-columns: minmax(0, 180px) auto;
  gap: 0.38rem;
  align-items: center;
}

.inline-edit-row {
  display: grid;
  grid-template-columns: 1fr auto auto;
  align-items: center;
  gap: 0.38rem;
}

.entry-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.entry-list li {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-1) 8%);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
  border-radius: 10px;
  padding: 0.64rem 0.7rem;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 0.7rem;
}

.team-row {
  position: relative;
  padding-right: 5.2rem;
}

.list-main {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 0.28rem;
}

.entry-title {
  font-weight: 800;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.team-meta-row {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.6rem;
}

.team-balance-row {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.34rem;
}

.team-balance-pill {
  border-radius: 999px;
  padding: 0.14rem 0.45rem;
  font-size: 0.78rem;
  font-weight: 700;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  background: color-mix(in srgb, var(--card) 92%, #f2f7ff 8%);
}

.team-balance-pill.ok {
  border-color: color-mix(in srgb, #1ea672 52%, var(--line) 48%);
  background: color-mix(in srgb, #1ea672 14%, var(--card) 86%);
}

.team-balance-pill.missing {
  border-color: color-mix(in srgb, #e0a100 56%, var(--line) 44%);
  background: color-mix(in srgb, #e0a100 16%, var(--card) 84%);
}

.team-balance-pill.excess {
  border-color: color-mix(in srgb, #d2555d 58%, var(--line) 42%);
  background: color-mix(in srgb, #d2555d 14%, var(--card) 86%);
}

.team-balance-note {
  margin: 0;
  font-size: 0.84rem;
}

.team-actions {
  position: absolute;
  top: 0.52rem;
  right: 0.52rem;
  display: flex;
  gap: 0.32rem;
}

.team-player-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.36rem;
}

.team-player-item {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.52rem;
  padding: 0.48rem 0.58rem;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-2) 10%);
  background: color-mix(in srgb, var(--card) 92%, #eef5ff 8%);
}

.team-player-main {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 0.44rem;
  flex: 1;
  padding-block: 0.14rem;
}

.team-player-name {
  font-size: 1.04rem;
  font-weight: 700;
  line-height: 1;
  overflow-wrap: anywhere;
}

.team-player-role {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  font-size: 0.95rem;
  line-height: 1;
  color: var(--ink-2);
}

.team-role-icon {
  font-size: 1.06rem;
}

.team-player-rank-icon {
  width: 24px;
  height: 24px;
  object-fit: contain;
  align-self: center;
}

.team-player-remove {
  margin-left: auto;
}

.team-player-remove.icon-btn {
  min-width: 2rem;
  min-height: 2rem;
  width: 2rem;
  height: 2rem;
  padding: 0.22rem;
}

.team-player-remove .material-symbols-rounded {
  font-size: 1.05rem;
}

.team-player-empty {
  font-size: 0.9rem;
}

.team-assign-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.4rem;
}

.team-assign-row {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.4rem;
  align-items: center;
}

.team-assign-match-count {
  margin: 0;
}

.team-assign-dropdown {
  position: absolute;
  top: calc(100% + 0.34rem);
  left: 0;
  right: auto;
  z-index: 24;
  width: max-content;
  min-width: min(26rem, 100%);
  max-width: min(42rem, calc(100vw - 2rem));
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-2) 16%);
  background: color-mix(in srgb, var(--card) 96%, #eef5ff 4%);
  border-radius: 10px;
  box-shadow: 0 10px 24px rgba(16, 39, 82, 0.18);
  padding: 0.45rem;
  display: grid;
  gap: 0.35rem;
}

.team-assign-results {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.32rem;
  justify-items: start;
  max-height: 14rem;
  overflow: auto;
}

/* Reset nested list item styles inherited from .entry-list li. */
.team-assign-results li {
  border: 0;
  background: transparent;
  border-radius: 0;
  padding: 0;
  display: block;
}

.team-assign-result-btn {
  width: auto;
  max-width: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  flex-wrap: wrap;
  gap: 0.35rem;
}

.team-assign-main {
  font-weight: 700;
}

.team-assign-notice {
  color: var(--ink-2);
  font-size: 0.82rem;
  margin-left: 1.58rem;
}

.team-assign-limit-note {
  margin: 0;
}

.team-assign-row .team-player-empty {
  margin: 0;
}

@media (max-width: 1100px) {
  .team-assign-row {
    grid-template-columns: 1fr;
  }

  .team-assign-dropdown {
    left: 0;
    right: 0;
    min-width: 0;
    max-width: none;
  }
}

@media (max-width: 900px) {
  .team-player-main {
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .quick-assign-item {
    grid-template-columns: 1fr;
  }

  .quick-assign-controls {
    grid-template-columns: 1fr;
  }
}
</style>
