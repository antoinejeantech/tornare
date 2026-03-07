<script setup>
import { computed, inject, reactive, ref } from 'vue'
import { getRankElo } from '../../lib/ranks'

const ctx = inject('eventCtx')
const assignmentSearchByTeam = reactive({})
const quickAssignTeamByPlayer = reactive({})
const quickAssignSearch = ref('')

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

  return ctx.event.players
    .filter((player) => player.team_id === teamId)
    .sort((a, b) => a.name.localeCompare(b.name))
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
    return players
  }

  return players.filter((player) => {
    return playerMatchesTokens(player, tokens)
  })
}

function visibleTeamAssignResults(teamId) {
  return filteredPlayersAssignableToTeam(teamId).slice(0, 10)
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
      <p class="muted">Creates one team per unassigned player.</p>
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
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.4rem;
  align-items: center;
}

.team-assign-match-count {
  margin: 0;
}

.team-assign-results {
  grid-column: 1 / -1;
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.32rem;
}

.team-assign-result-btn {
  width: 100%;
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
  grid-column: 1 / -1;
  margin: 0;
}

.team-assign-row .team-player-empty {
  grid-column: 1 / -1;
  margin: 0;
}

@media (max-width: 1100px) {
  .team-assign-row {
    grid-template-columns: 1fr;
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
