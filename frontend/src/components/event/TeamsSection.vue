<script setup>
import { computed, inject } from 'vue'
import { getRankElo } from '../../lib/ranks'

const ctx = inject('eventCtx')

const unassignedPlayersCount = computed(() => {
  if (!ctx.event) {
    return 0
  }

  return ctx.event.players.filter((player) => !player.team_id).length
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

  return ctx.event.players.filter((player) => player.team_id !== teamId)
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
            <button
              v-for="player in playersAssignableToTeam(team.id)"
              :key="`assign-${team.id}-${player.id}`"
              class="btn-secondary team-assign-btn"
              :disabled="Boolean(ctx.savingPlayerTeams[player.id])"
              @click="ctx.assignPlayerToTeam(player.id, team.id)"
            >
              <span class="material-symbols-rounded" aria-hidden="true">
                {{ ctx.savingPlayerTeams[player.id] ? 'hourglass_top' : 'person_add' }}
              </span>
              <span class="team-assign-main">{{ player.name }} · {{ player.role }} · {{ player.rank }}</span>
              <span v-if="assignmentNotice(player)" class="team-assign-notice">{{ assignmentNotice(player) }}</span>
            </button>
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

.team-assign-btn {
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.35rem;
  flex-wrap: wrap;
}

.team-assign-main {
  font-weight: 700;
}

.team-assign-notice {
  color: var(--ink-2);
  font-size: 0.82rem;
  margin-left: 1.58rem;
}

@media (max-width: 900px) {
  .team-player-main {
    flex-wrap: wrap;
    gap: 0.3rem;
  }
}
</style>
