<script setup>
import { inject, ref } from 'vue'
import { useRouter } from 'vue-router'

const ctx = inject('eventCtx')
const router = useRouter()
const showCreateMatchForm = ref(false)
const editingMatchups = ref({})
const teamPalette = ['#f04f23', '#0f2f8c', '#00a3a3', '#7828c8', '#f7b801', '#2e7d4f']

function colorForTeamId(teamId) {
  if (!teamId) {
    return '#c8ccda'
  }

  const id = String(teamId)
  let hash = 0
  for (let index = 0; index < id.length; index += 1) {
    hash = (hash * 31 + id.charCodeAt(index)) >>> 0
  }

  return teamPalette[hash % teamPalette.length]
}

function matchStripeStyle(match) {
  return {
    '--team-a': colorForTeamId(match.team_a_id),
    '--team-b': colorForTeamId(match.team_b_id),
  }
}

function openMatch(matchId) {
  router.push({ name: 'match', params: { id: matchId } })
}

function isEditingMatchup(matchId) {
  return Boolean(editingMatchups.value[matchId])
}

function toggleMatchupEditor(matchId) {
  editingMatchups.value = {
    ...editingMatchups.value,
    [matchId]: !editingMatchups.value[matchId],
  }
}

function closeMatchupEditor(matchId) {
  editingMatchups.value = {
    ...editingMatchups.value,
    [matchId]: false,
  }
}

function toggleCreateMatchForm() {
  showCreateMatchForm.value = !showCreateMatchForm.value
}

async function submitCreateMatch() {
  await ctx.createMatch()
  if (!ctx.creatingMatch && !ctx.newMatchTitle && !ctx.newMatchMap) {
    showCreateMatchForm.value = false
  }
}

function selectionForMatch(matchId) {
  const fallback = { teamAId: '', teamBId: '' }
  const selection = ctx.matchupSelections?.[matchId]
  if (!selection) {
    return fallback
  }

  return {
    teamAId: String(selection.teamAId || ''),
    teamBId: String(selection.teamBId || ''),
  }
}

function teamASelection(matchId) {
  return selectionForMatch(matchId).teamAId
}

function teamBSelection(matchId) {
  return selectionForMatch(matchId).teamBId
}

function hasDuplicateTeamSelection(match) {
  const selection = selectionForMatch(match.id)
  return Boolean(selection.teamAId && selection.teamAId === selection.teamBId)
}

function hasCompleteMatchup(match) {
  const selection = selectionForMatch(match.id)
  return Boolean(selection.teamAId && selection.teamBId)
}

function matchupChanged(match) {
  const selection = selectionForMatch(match.id)
  const existingA = String(match.team_a_id || '')
  const existingB = String(match.team_b_id || '')
  return selection.teamAId !== existingA || selection.teamBId !== existingB
}

function canSaveMatchup(match) {
  if (!ctx.canManageEvent || Boolean(ctx.savingMatchups[match.id])) {
    return false
  }

  if (hasDuplicateTeamSelection(match) || !hasCompleteMatchup(match)) {
    return false
  }

  return matchupChanged(match)
}

function matchupStateLabel(match) {
  if (hasDuplicateTeamSelection(match)) {
    return 'Choose two different teams'
  }

  if (hasCompleteMatchup(match)) {
    return matchupChanged(match) ? 'Matchup ready to save' : ''
  }

  return 'Matchup incomplete'
}

function showMatchupState(match) {
  return Boolean(matchupStateLabel(match))
}

function matchupStateClass(match) {
  if (hasDuplicateTeamSelection(match)) {
    return 'is-error'
  }

  if (hasCompleteMatchup(match)) {
    return matchupChanged(match) ? 'is-ready' : 'is-ok'
  }

  return 'is-muted'
}

function teamNameById(teamId) {
  const teams = Array.isArray(ctx.event?.teams) ? ctx.event.teams : []
  const team = teams.find((entry) => String(entry.id) === String(teamId || ''))
  return team?.name || 'TBD'
}

function currentMatchupLabel(match, slot) {
  const selection = selectionForMatch(match.id)
  const selectedId = slot === 'A' ? selection.teamAId : selection.teamBId
  if (selectedId) {
    return teamNameById(selectedId)
  }

  return slot === 'A' ? (match.team_a_name || 'TBD') : (match.team_b_name || 'TBD')
}

async function saveMatchupAndClose(match) {
  const saved = await ctx.saveMatchup(match.id)
  if (saved) {
    closeMatchupEditor(match.id)
  }
}
</script>

<template>
  <div>
    <div v-if="ctx.canManageEvent" class="match-create-panel">
      <button class="btn-secondary" type="button" @click="toggleCreateMatchForm">
        {{ showCreateMatchForm ? 'Close match form' : 'New match' }}
      </button>

      <form v-if="showCreateMatchForm" class="grid-form" @submit.prevent="submitCreateMatch">
        <label>
          Match title
          <input v-model="ctx.newMatchTitle" placeholder="Match 1" />
        </label>
        <label>
          Map
          <input v-model="ctx.newMatchMap" placeholder="King's Row" />
        </label>
        <button type="submit" class="btn-primary" :disabled="!ctx.canCreateMatch || ctx.creatingMatch">
          {{ ctx.creatingMatch ? 'Creating...' : 'Create match' }}
        </button>
      </form>
    </div>

    <p v-if="ctx.event.matches.length === 0" class="muted">No matches yet. Create your first match.</p>
    <ul v-else class="entry-list match-list-compact">
      <li
        v-for="match in ctx.event.matches"
        :key="match.id"
        class="match-item match-item-openable"
        :class="{ 'matchup-set': Boolean(match.team_a_id && match.team_b_id) }"
        :style="matchStripeStyle(match)"
        role="button"
        tabindex="0"
        @click="openMatch(match.id)"
        @keydown.enter.prevent="openMatch(match.id)"
        @keydown.space.prevent="openMatch(match.id)"
      >
        <div class="match-primary">
          <span class="entry-title">{{ match.title }}</span>
          <span class="muted match-map-meta">{{ match.map }} · {{ match.players.length }}/{{ match.max_players }}</span>
        </div>

        <div class="list-main">
          <div v-if="!(ctx.canManageEvent && isEditingMatchup(match.id))" class="matchup-summary-row" @click.stop>
            <span class="matchup-team-pill">{{ currentMatchupLabel(match, 'A') }}</span>
            <span class="muted">vs</span>
            <span class="matchup-team-pill">{{ currentMatchupLabel(match, 'B') }}</span>
            <span v-if="showMatchupState(match)" class="matchup-state" :class="matchupStateClass(match)">{{ matchupStateLabel(match) }}</span>
          </div>

          <div v-else-if="ctx.canManageEvent && isEditingMatchup(match.id)" class="matchup-row" @click.stop>
            <select
              v-model="ctx.matchupSelections[match.id].teamAId"
              :disabled="Boolean(ctx.savingMatchups[match.id])"
            >
              <option value="">Choose team</option>
              <option
                v-for="team in ctx.event.teams"
                :key="`a-${team.id}`"
                :value="String(team.id)"
                :disabled="String(team.id) === teamBSelection(match.id)"
              >
                {{ team.name }}
              </option>
            </select>
            <span class="muted">vs</span>
            <select
              v-model="ctx.matchupSelections[match.id].teamBId"
              :disabled="Boolean(ctx.savingMatchups[match.id])"
            >
              <option value="">Choose team</option>
              <option
                v-for="team in ctx.event.teams"
                :key="`b-${team.id}`"
                :value="String(team.id)"
                :disabled="String(team.id) === teamASelection(match.id)"
              >
                {{ team.name }}
              </option>
            </select>
            <button
              class="btn-secondary icon-btn"
              :disabled="!canSaveMatchup(match)"
              :title="ctx.savingMatchups[match.id] ? 'Saving matchup' : (canSaveMatchup(match) ? 'Save matchup' : 'No matchup changes to save')"
              @click="saveMatchupAndClose(match)"
            >
              <span class="material-symbols-rounded" aria-hidden="true">
                {{ ctx.savingMatchups[match.id] ? 'hourglass_top' : 'save' }}
              </span>
              <span class="sr-only">{{ ctx.savingMatchups[match.id] ? 'Saving matchup' : 'Save matchup' }}</span>
            </button>
            <button class="btn-secondary" type="button" @click="closeMatchupEditor(match.id)">
              Cancel
            </button>
          </div>
        </div>
        <div class="match-side-actions" @click.stop>
          <button
            v-if="ctx.canManageEvent"
            class="btn-secondary icon-btn"
            :title="isEditingMatchup(match.id) ? 'Close matchup editor' : 'Edit matchup'"
            @click="toggleMatchupEditor(match.id)"
          >
            <span class="material-symbols-rounded" aria-hidden="true">
              {{ isEditingMatchup(match.id) ? 'close' : 'edit' }}
            </span>
            <span class="sr-only">{{ isEditingMatchup(match.id) ? 'Close matchup editor' : 'Edit matchup' }}</span>
          </button>
          <button
            v-if="ctx.canManageEvent"
            class="btn-danger icon-btn"
            :disabled="ctx.deletingMatchId === match.id"
            :title="ctx.deletingMatchId === match.id ? 'Deleting match' : 'Delete match'"
            @click="ctx.deleteMatch(match.id)"
          >
            <span class="material-symbols-rounded" aria-hidden="true">
              {{ ctx.deletingMatchId === match.id ? 'hourglass_top' : 'delete' }}
            </span>
            <span class="sr-only">{{ ctx.deletingMatchId === match.id ? 'Deleting match' : 'Delete match' }}</span>
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.grid-form {
  display: grid;
  gap: 0.56rem;
}

.match-create-panel {
  display: grid;
  gap: 0.56rem;
  margin-bottom: 0.72rem;
}

.grid-form label {
  display: grid;
  gap: 0.28rem;
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
  border-radius: 12px;
  padding: 1rem 1.05rem;
  display: grid;
  grid-template-columns: minmax(180px, 0.9fr) minmax(0, 1.35fr) auto;
  align-items: center;
  gap: 1rem;
}

.match-primary {
  min-width: 0;
  display: grid;
  gap: 0.32rem;
}

.match-map-meta {
  font-size: 0.95rem;
  line-height: 1.25;
}

.match-list-compact {
  max-height: 420px;
  overflow: auto;
  padding-right: 0.15rem;
}

.match-item {
  position: relative;
  align-items: flex-start;
  border-left: 4px solid var(--line);
}

.match-item-openable {
  cursor: pointer;
  transition: box-shadow 0.15s ease, border-color 0.15s ease;
}

.match-item-openable:hover,
.match-item-openable:focus-visible {
  box-shadow:
    0 12px 24px rgba(16, 34, 72, 0.16),
    0 3px 9px rgba(16, 34, 72, 0.14);
  border-color: color-mix(in srgb, var(--brand-2) 40%, var(--line) 60%);
  outline: none;
}

.match-item.matchup-set {
  border-left-color: transparent;
}

.match-item.matchup-set::before {
  content: "";
  position: absolute;
  left: -1px;
  top: 0;
  bottom: 0;
  width: 6px;
  border-radius: 10px 0 0 10px;
  background: linear-gradient(180deg, var(--team-a), var(--team-b));
}

.list-main {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 0.5rem;
}

.entry-title {
  font-weight: 800;
  font-size: 1.12rem;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.matchup-row {
  display: flex;
  align-items: center;
  gap: 0.66rem;
  flex-wrap: wrap;
}

.matchup-summary-row {
  display: flex;
  align-items: center;
  gap: 0.62rem;
  flex-wrap: wrap;
}

.matchup-team-pill {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 0.26rem 0.74rem;
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-2) 16%);
  background: color-mix(in srgb, var(--card) 92%, #1a2740 8%);
  font-size: 0.92rem;
  font-weight: 700;
}

.matchup-row select {
  min-height: 2.7rem;
  font-size: 1.02rem;
}

.matchup-row .btn-secondary,
.matchup-summary-row .btn-secondary {
  min-height: 2.55rem;
  padding-inline: 0.9rem;
  font-size: 0.95rem;
}

.matchup-state {
  margin-left: 0.2rem;
  border-radius: 999px;
  padding: 0.24rem 0.7rem;
  font-size: 0.88rem;
  font-weight: 700;
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-2) 16%);
  background: color-mix(in srgb, var(--card) 92%, #1a2740 8%);
}

.matchup-state.is-muted {
  color: var(--ink-2);
}

.matchup-state.is-ready {
  color: #d5f6e7;
  background: color-mix(in srgb, #1d8e61 22%, var(--card) 78%);
  border-color: color-mix(in srgb, #25b177 56%, var(--line) 44%);
}

.matchup-state.is-error {
  color: #ffd8cf;
  background: color-mix(in srgb, #a8412f 22%, var(--card) 78%);
  border-color: color-mix(in srgb, #cf5f4a 58%, var(--line) 42%);
}

.match-side-actions {
  display: grid;
  gap: 0.45rem;
  align-items: center;
}

.match-side-actions .icon-btn {
  min-width: 2.45rem;
  min-height: 2.45rem;
}

@media (max-width: 900px) {
  .match-item {
    grid-template-columns: 1fr;
    align-items: stretch;
  }

  .match-side-actions {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
