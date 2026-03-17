<script setup>
import { computed, inject, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { getDateTimestamp, isoToDatetimeLocalValue } from '../../lib/dates'
import { sortPlayersByRoleThenName } from '../../lib/roles'
import PlayerNameplate from '../player/PlayerNameplate.vue'

const ctx = inject('eventCtx')

// ── Modal state ───────────────────────────────────────────────────────────────
const activeMatchId = ref(null)
const showCreateForm = ref(false)

const activeMatch = computed(() =>
  activeMatchId.value == null
    ? null
    : (ctx.event?.matches ?? []).find((m) => m.id === activeMatchId.value) ?? null
)

const playersA = computed(() => {
  const m = activeMatch.value
  if (!m?.team_a_id) return []
  return sortPlayersByRoleThenName(m.players.filter((p) => p.team_id === m.team_a_id))
})

const playersB = computed(() => {
  const m = activeMatch.value
  if (!m?.team_b_id) return []
  return sortPlayersByRoleThenName(m.players.filter((p) => p.team_id === m.team_b_id))
})

const playersUnassigned = computed(() => {
  const m = activeMatch.value
  if (!m) return []
  const assigned = new Set([
    ...(m.team_a_id ? [m.team_a_id] : []),
    ...(m.team_b_id ? [m.team_b_id] : []),
  ])
  return sortPlayersByRoleThenName(m.players.filter((p) => !assigned.has(p.team_id)))
})

const hasTeamRosters = computed(() => Boolean(activeMatch.value?.team_a_id && activeMatch.value?.team_b_id))

function openModal(matchId) {
  activeMatchId.value = matchId
}

function closeModal() {
  activeMatchId.value = null
}

function onOverlayClick(e) {
  if (e.target === e.currentTarget) closeModal()
}

function closeCreateForm() {
  showCreateForm.value = false
}

function onCreateOverlayClick(e) {
  if (e.target === e.currentTarget) closeCreateForm()
}

function onEscapeKey(e) {
  if (e.key !== 'Escape') return
  if (activeMatchId.value !== null) closeModal()
  else if (showCreateForm.value) closeCreateForm()
}

onMounted(() => document.addEventListener('keydown', onEscapeKey))
onBeforeUnmount(() => document.removeEventListener('keydown', onEscapeKey))

// ── Team color stripe ─────────────────────────────────────────────────────────
const teamPalette = ['#f04f23', '#0f2f8c', '#00a3a3', '#7828c8', '#f7b801', '#2e7d4f']

function colorForTeamId(teamId) {
  if (!teamId) return '#c8ccda'
  const id = String(teamId)
  let hash = 0
  for (let i = 0; i < id.length; i++) hash = (hash * 31 + id.charCodeAt(i)) >>> 0
  return teamPalette[hash % teamPalette.length]
}

function matchStripeStyle(match) {
  return {
    '--stripe-a': colorForTeamId(match.team_a_id),
    '--stripe-b': colorForTeamId(match.team_b_id),
  }
}

// ── Matchup helpers ───────────────────────────────────────────────────────────
function selectionFor(matchId) {
  const s = ctx.matchupSelections?.[matchId]
  return { teamAId: String(s?.teamAId || ''), teamBId: String(s?.teamBId || '') }
}

function isDuplicateSelection(matchId) {
  const { teamAId, teamBId } = selectionFor(matchId)
  return Boolean(teamAId && teamAId === teamBId)
}

function isCompleteSelection(matchId) {
  const { teamAId, teamBId } = selectionFor(matchId)
  return Boolean(teamAId && teamBId)
}

function isSelectionChanged(match) {
  const { teamAId, teamBId } = selectionFor(match.id)
  return teamAId !== String(match.team_a_id || '') || teamBId !== String(match.team_b_id || '')
}

function canSaveMatchup(match) {
  if (!ctx.canManageEvent || ctx.savingMatchups[match.id]) return false
  if (isDuplicateSelection(match.id) || !isCompleteSelection(match.id)) return false
  return isSelectionChanged(match)
}

function matchupHint(matchId) {
  if (isDuplicateSelection(matchId)) return 'Choose two different teams'
  return ''
}

// ── Match status ──────────────────────────────────────────────────────────────
function matchStatus(match) {
  if (match.winner_team_id) return 'done'
  if (match.team_a_id && match.team_b_id) return 'ready'
  return 'open'
}

const STATUS_LABELS = { done: 'Done', ready: 'Ready', open: 'Open' }

// ── Create match ──────────────────────────────────────────────────────────────
function toggleCreateForm() {
  showCreateForm.value = !showCreateForm.value
}

async function submitCreateMatch() {
  await ctx.createMatch()
  if (!ctx.creatingMatch && !ctx.newMatchTitle && !ctx.newMatchMap) {
    showCreateForm.value = false
  }
}

// ── Delete match ──────────────────────────────────────────────────────────────
async function deleteActiveMatch() {
  if (!activeMatch.value) return
  const id = activeMatch.value.id
  closeModal()
  await ctx.deleteMatch(id)
}

// ── Winner reporting ──────────────────────────────────────────────────────────
async function reportWinner(matchId, teamId) {
  await ctx.reportMatchWinner(matchId, teamId)
}

async function cancelWinner() {
  if (!activeMatch.value) return
  await ctx.cancelMatchWinner(activeMatch.value.id)
}

// ── Stats ─────────────────────────────────────────────────────────────────────
const stats = computed(() => {
  const matches = ctx.event?.matches ?? []
  return {
    total: matches.length,
    played: matches.filter((m) => Boolean(m.winner_team_id)).length,
    players: ctx.event?.players?.length ?? 0,
    teams: ctx.event?.teams?.length ?? 0,
  }
})

const sortedMatches = computed(() => {
  const matches = ctx.event?.matches ?? []
  const now = Date.now()
  const withDate = matches
    .filter((m) => m.start_date)
    .map((m) => ({ m, ts: getDateTimestamp(m.start_date) }))
    .filter((entry) => entry.ts !== null)
    .sort((a, b) => {
      const aFuture = a.ts >= now
      const bFuture = b.ts >= now
      if (aFuture !== bFuture) return aFuture ? -1 : 1
      return aFuture ? a.ts - b.ts : b.ts - a.ts
    })
    .map((x) => x.m)
  const withoutDate = matches.filter((m) => !m.start_date || getDateTimestamp(m.start_date) === null)
  return [...withDate, ...withoutDate]
})

defineExpose({ toggleCreateForm })

// ── Schedule editing ─────────────────────────────────────────────────────────
const editStartDate = ref('')
const savingStartDate = ref(false)

watch(() => activeMatchId.value, (newId) => {
  if (newId != null) {
    const m = (ctx.event?.matches ?? []).find((m) => m.id === newId)
    editStartDate.value = m?.start_date ? isoToDatetimeLocalValue(m.start_date) : ''
  }
})

function formatMatchStartDate(isoStr) {
  const d = new Date(isoStr)
  if (isNaN(d.getTime())) return isoStr
  return d.toLocaleString(undefined, {
    year: 'numeric', month: 'short', day: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}

async function saveStartDate() {
  if (!activeMatch.value || savingStartDate.value) return
  savingStartDate.value = true
  try {
    await ctx.updateMatchStartDate(activeMatch.value.id, editStartDate.value || null)
  } finally {
    savingStartDate.value = false
  }
}
</script>

<template>
  <div class="pug-root" style="min-width: 0">

    <!-- ── New match modal ────────────────────────────────────────────────── -->
    <Teleport to="body">
      <div
        v-if="ctx.canManageEvent && showCreateForm"
        class="pug-modal-overlay"
        @click="onCreateOverlayClick"
      >
        <div class="pug-modal pug-create-modal" role="dialog" aria-modal="true" aria-labelledby="pug-create-title">
          <div class="pug-modal-header pug-create-modal-header">
            <div class="modal-header-content">
              <h2 id="pug-create-title" class="modal-title">New Match</h2>
            </div>
            <button class="btn-secondary icon-btn modal-close-btn" title="Close" @click="closeCreateForm">
              <span class="material-symbols-rounded" aria-hidden="true">close</span>
              <span class="sr-only">Close</span>
            </button>
          </div>
          <div class="pug-modal-body">
            <form class="grid-form pug-create-form" @submit.prevent="submitCreateMatch">
              <label>
                Title
                <input v-model="ctx.newMatchTitle" placeholder="Match 1" />
              </label>
              <label>
                Map
                <input v-model="ctx.newMatchMap" placeholder="King's Row" />
              </label>
              <template v-if="ctx.event?.teams?.length > 0">
                <div class="pug-create-teams-row">
                  <label>
                    Team A
                    <select v-model="ctx.newMatchTeamAId">
                      <option value="">None</option>
                      <option
                        v-for="team in ctx.event.teams"
                        :key="`ca-${team.id}`"
                        :value="String(team.id)"
                        :disabled="String(team.id) === String(ctx.newMatchTeamBId)"
                      >{{ team.name }}</option>
                    </select>
                  </label>
                  <span class="pug-create-vs" aria-hidden="true">VS</span>
                  <label>
                    Team B
                    <select v-model="ctx.newMatchTeamBId">
                      <option value="">None</option>
                      <option
                        v-for="team in ctx.event.teams"
                        :key="`cb-${team.id}`"
                        :value="String(team.id)"
                        :disabled="String(team.id) === String(ctx.newMatchTeamAId)"
                      >{{ team.name }}</option>
                    </select>
                  </label>
                </div>
              </template>
              <label>
                Start Date <span class="form-label-hint">(optional)</span>
                <input type="datetime-local" v-model="ctx.newMatchStartDate" />
              </label>
              <div class="pug-create-footer-section">
                <button
                  type="submit"
                  class="btn-primary icon-btn"
                  :disabled="!ctx.canCreateMatch || ctx.creatingMatch"
                >
                  <span class="material-symbols-rounded" aria-hidden="true">
                    {{ ctx.creatingMatch ? 'hourglass_top' : 'save' }}
                  </span>
                  {{ ctx.creatingMatch ? 'Saving...' : 'Save' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- ── Stats bar ──────────────────────────────────────────────────────── -->
    <div v-if="stats.total > 0" class="pug-stats-bar">
      <div class="pug-stat-item">
        <span class="pug-stat-value">{{ stats.players }}</span>
        <span class="pug-stat-label">Total Players</span>
      </div>
      <div class="pug-stat-item">
        <span class="pug-stat-value">{{ stats.teams }}</span>
        <span class="pug-stat-label">Teams</span>
      </div>
      <div class="pug-stat-item">
        <span class="pug-stat-value">{{ stats.total }}</span>
        <span class="pug-stat-label">Total Matches</span>
      </div>
      <div class="pug-stat-item">
        <span class="pug-stat-value">
          {{ stats.played }}<span class="pug-stat-of muted">/ {{ stats.total }}</span>
        </span>
        <span class="pug-stat-label">Matches Played</span>
      </div>
    </div>

    <!-- ── Match grid ──────────────────────────────────────────────────────── -->
    <p v-if="!ctx.event?.matches?.length" class="pug-empty">
      No matches yet.{{ ctx.canManageEvent ? ' Create your first match above.' : '' }}
    </p>
    <ul v-else class="pug-match-grid" role="list">
      <li
        v-for="match in sortedMatches"
        :key="match.id"
        class="pug-match-card"
        :class="`is-${matchStatus(match)}`"
        :style="matchStripeStyle(match)"
        role="button"
        tabindex="0"
        @click="openModal(match.id)"
        @keydown.enter.prevent="openModal(match.id)"
        @keydown.space.prevent="openModal(match.id)"
      >
        <div class="match-card-stripe" aria-hidden="true"></div>
        <div class="match-card-body">
          <div class="match-card-header">
            <span class="match-card-title">{{ match.title }}</span>
            <span class="match-card-badge" :class="`badge-${matchStatus(match)}`">
              {{ STATUS_LABELS[matchStatus(match)] }}
            </span>
          </div>
          <div class="match-card-matchup">
            <div
              class="match-card-team-row"
              :class="{ 'is-winner': match.winner_team_id && match.winner_team_id === match.team_a_id }"
            >
              <span
                v-if="match.winner_team_id && match.winner_team_id === match.team_a_id"
                class="material-symbols-rounded match-team-trophy"
                aria-hidden="true"
              >emoji_events</span>
              {{ match.team_a_name || 'TBD' }}
            </div>
            <div class="match-card-vs-divider" aria-hidden="true">
              <span class="vs-badge">VS</span>
            </div>
            <div
              class="match-card-team-row"
              :class="{ 'is-winner': match.winner_team_id && match.winner_team_id === match.team_b_id }"
            >
              <span
                v-if="match.winner_team_id && match.winner_team_id === match.team_b_id"
                class="material-symbols-rounded match-team-trophy"
                aria-hidden="true"
              >emoji_events</span>
              {{ match.team_b_name || 'TBD' }}
            </div>
          </div>
          <div class="match-card-footer">
            <div class="match-card-footer-row">
              <span class="match-card-meta">
                <span class="material-symbols-rounded" aria-hidden="true">map</span>
                {{ match.map }}
              </span>
              <span class="match-card-meta">
                <span class="material-symbols-rounded" aria-hidden="true">group</span>
                {{ match.players.length }}/{{ match.max_players }}
              </span>
            </div>
            <div v-if="match.start_date" class="match-card-footer-row match-card-date-row">
              <span class="match-card-meta">
                <span class="material-symbols-rounded" aria-hidden="true">schedule</span>
                {{ formatMatchStartDate(match.start_date) }}
              </span>
            </div>
          </div>
        </div>
      </li>
    </ul>

    <!-- ── Match detail modal ─────────────────────────────────────────────── -->
    <Teleport to="body">
      <div
        v-if="activeMatch"
        class="pug-modal-overlay"
        @click="onOverlayClick"
      >
        <div
          class="pug-modal"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="`pug-modal-title-${activeMatch.id}`"
        >
          <!-- Header (fixed) -->
          <div class="pug-modal-header" :style="matchStripeStyle(activeMatch)">
            <div class="modal-header-stripe" aria-hidden="true"></div>
            <div class="modal-header-content">
              <div class="modal-title-row">
                <h2 :id="`pug-modal-title-${activeMatch.id}`" class="modal-title">{{ activeMatch.title }}</h2>
                <span class="match-card-badge" :class="`badge-${matchStatus(activeMatch)}`">
                  {{ STATUS_LABELS[matchStatus(activeMatch)] }}
                </span>
              </div>
              <p class="modal-subtitle">
                <span class="modal-meta-item">
                  <span class="material-symbols-rounded modal-meta-icon" aria-hidden="true">map</span>
                  {{ activeMatch.map }}
                </span>
                <span class="modal-meta-sep" aria-hidden="true">·</span>
                <span class="modal-meta-item">
                  <span class="material-symbols-rounded modal-meta-icon" aria-hidden="true">group</span>
                  {{ activeMatch.players.length }}/{{ activeMatch.max_players }} players
                </span>
                <template v-if="activeMatch.start_date">
                  <span class="modal-meta-sep" aria-hidden="true">·</span>
                  <span class="modal-meta-item">
                    <span class="material-symbols-rounded modal-meta-icon" aria-hidden="true">schedule</span>
                    {{ formatMatchStartDate(activeMatch.start_date) }}
                  </span>
                </template>
              </p>
            </div>
            <button class="btn-secondary icon-btn modal-close-btn" title="Close" @click="closeModal">
              <span class="material-symbols-rounded" aria-hidden="true">close</span>
              <span class="sr-only">Close</span>
            </button>
          </div>

          <!-- Scrollable body -->
          <div class="pug-modal-body">

            <!-- Schedule section -->
            <div v-if="ctx.canManageEvent" class="modal-section">
              <h3 class="modal-section-title">Schedule</h3>
              <div class="schedule-editor-row">
                <input
                  type="datetime-local"
                  v-model="editStartDate"
                  :disabled="savingStartDate"
                  class="schedule-date-input"
                />
                <button
                  class="btn-primary icon-btn"
                  :disabled="savingStartDate"
                  @click="saveStartDate"
                >
                  <span class="material-symbols-rounded" aria-hidden="true">
                    {{ savingStartDate ? 'hourglass_top' : 'save' }}
                  </span>
                  {{ savingStartDate ? 'Saving...' : 'Save' }}
                </button>
              </div>
            </div>

            <!-- Matchup section -->
            <div class="modal-section">
              <h3 class="modal-section-title">Matchup</h3>
              <template v-if="ctx.canManageEvent">
                <div class="matchup-editor-row">
                  <select
                    v-if="ctx.matchupSelections[activeMatch.id]"
                    v-model="ctx.matchupSelections[activeMatch.id].teamAId"
                    :disabled="Boolean(ctx.savingMatchups[activeMatch.id])"
                  >
                    <option value="">Choose team A</option>
                    <option
                      v-for="team in ctx.event.teams"
                      :key="`a-${team.id}`"
                      :value="String(team.id)"
                      :disabled="String(team.id) === selectionFor(activeMatch.id).teamBId"
                    >{{ team.name }}</option>
                  </select>
                  <span class="vs-sep" aria-hidden="true">vs</span>
                  <select
                    v-if="ctx.matchupSelections[activeMatch.id]"
                    v-model="ctx.matchupSelections[activeMatch.id].teamBId"
                    :disabled="Boolean(ctx.savingMatchups[activeMatch.id])"
                  >
                    <option value="">Choose team B</option>
                    <option
                      v-for="team in ctx.event.teams"
                      :key="`b-${team.id}`"
                      :value="String(team.id)"
                      :disabled="String(team.id) === selectionFor(activeMatch.id).teamAId"
                    >{{ team.name }}</option>
                  </select>
                  <button
                    class="btn-primary icon-btn"
                    :disabled="!canSaveMatchup(activeMatch)"
                    @click="ctx.saveMatchup(activeMatch.id)"
                  >
                    <span class="material-symbols-rounded" aria-hidden="true">
                      {{ ctx.savingMatchups[activeMatch.id] ? 'hourglass_top' : 'save' }}
                    </span>
                    {{ ctx.savingMatchups[activeMatch.id] ? 'Saving...' : 'Save' }}
                  </button>
                </div>
                <p v-if="matchupHint(activeMatch.id)" class="matchup-hint is-error">
                  {{ matchupHint(activeMatch.id) }}
                </p>
              </template>
              <div v-else class="matchup-display-row">
                <span class="team-chip">{{ activeMatch.team_a_name || 'TBD' }}</span>
                <span class="vs-sep" aria-hidden="true">vs</span>
                <span class="team-chip">{{ activeMatch.team_b_name || 'TBD' }}</span>
              </div>
            </div>

            <!-- Result section (only if matchup is set) -->
            <div v-if="activeMatch.team_a_id && activeMatch.team_b_id" class="modal-section">
              <h3 class="modal-section-title">Result</h3>
              <template v-if="!activeMatch.winner_team_id">
                <p v-if="!ctx.canManageEvent" class="modal-hint-text">No result declared yet.</p>
                <div v-else class="winner-declare-row">
                  <button
                    class="btn-secondary icon-btn winner-declare-btn"
                    :disabled="Boolean(ctx.reportingWinners[activeMatch.id])"
                    @click="reportWinner(activeMatch.id, activeMatch.team_a_id)"
                  >
                    <span class="material-symbols-rounded" aria-hidden="true">emoji_events</span>
                    {{ activeMatch.team_a_name || 'Team A' }} wins
                  </button>
                  <button
                    class="btn-secondary icon-btn winner-declare-btn"
                    :disabled="Boolean(ctx.reportingWinners[activeMatch.id])"
                    @click="reportWinner(activeMatch.id, activeMatch.team_b_id)"
                  >
                    <span class="material-symbols-rounded" aria-hidden="true">emoji_events</span>
                    {{ activeMatch.team_b_name || 'Team B' }} wins
                  </button>
                </div>
              </template>
              <div v-else class="winner-result-row">
                <div class="winner-result-label">
                  <span class="material-symbols-rounded winner-trophy" aria-hidden="true">emoji_events</span>
                  <strong>{{ activeMatch.winner_team_name || 'Unknown' }}</strong>
                  <span class="muted">won this match</span>
                </div>
                <button
                  v-if="ctx.canManageEvent"
                  class="btn-secondary"
                  :disabled="Boolean(ctx.cancellingWinners[activeMatch.id])"
                  @click="cancelWinner"
                >
                  {{ ctx.cancellingWinners[activeMatch.id] ? 'Cancelling...' : 'Cancel result' }}
                </button>
              </div>
            </div>

            <!-- Players section -->
            <div v-if="activeMatch.players.length > 0" class="modal-section">
              <h3 class="modal-section-title">Players ({{ activeMatch.players.length }})</h3>
              <!-- Two-column roster when teams are set -->
              <div v-if="hasTeamRosters" class="modal-roster-grid">
                <div class="modal-team-col">
                  <h4 class="modal-team-name">{{ activeMatch.team_a_name || 'Team A' }}</h4>
                  <p v-if="playersA.length === 0" class="modal-hint-text">No players assigned.</p>
                  <ul v-else class="modal-player-list">
                    <li v-for="player in playersA" :key="`a-${player.id}`" class="modal-player-row">
                      <PlayerNameplate :name="player.name" :role="player.role" :rank="player.rank" compact />
                    </li>
                  </ul>
                </div>
                <div class="modal-team-col">
                  <h4 class="modal-team-name">{{ activeMatch.team_b_name || 'Team B' }}</h4>
                  <p v-if="playersB.length === 0" class="modal-hint-text">No players assigned.</p>
                  <ul v-else class="modal-player-list">
                    <li v-for="player in playersB" :key="`b-${player.id}`" class="modal-player-row">
                      <PlayerNameplate :name="player.name" :role="player.role" :rank="player.rank" compact />
                    </li>
                  </ul>
                </div>
              </div>
              <!-- Flat list when no teams set -->
              <ul v-else class="modal-player-list">
                <li v-for="player in playersUnassigned" :key="player.id" class="modal-player-row">
                  <PlayerNameplate :name="player.name" :role="player.role" :rank="player.rank" compact />
                </li>
              </ul>
              <!-- Unassigned players when teams exist but some players have no team -->
              <template v-if="hasTeamRosters && playersUnassigned.length > 0">
                <h4 class="modal-team-name modal-team-name--unassigned">Unassigned</h4>
                <ul class="modal-player-list">
                  <li v-for="player in playersUnassigned" :key="`u-${player.id}`" class="modal-player-row">
                    <PlayerNameplate :name="player.name" :role="player.role" :rank="player.rank" compact />
                  </li>
                </ul>
              </template>
            </div>

            <!-- Admin — danger zone -->
            <div v-if="ctx.canManageEvent" class="modal-danger-zone">
              <button
                class="btn-danger icon-btn"
                :disabled="ctx.deletingMatchId === activeMatch.id"
                @click="deleteActiveMatch"
              >
                <span class="material-symbols-rounded" aria-hidden="true">
                  {{ ctx.deletingMatchId === activeMatch.id ? 'hourglass_top' : 'delete' }}
                </span>
                {{ ctx.deletingMatchId === activeMatch.id ? 'Deleting...' : 'Delete match' }}
              </button>
            </div>

          </div><!-- /.pug-modal-body -->
        </div><!-- /.pug-modal -->
      </div><!-- /.pug-modal-overlay -->
    </Teleport>

  </div>
</template>

<style scoped>
/* ── Root ────────────────────────────────────────────────────────────────── */
.pug-root {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

/* ── Create match modal ──────────────────────────────────────────────────── */
.pug-create-modal {
  max-width: 420px;
}

.pug-create-modal-header {
  padding: 0;
}

.pug-create-modal-header .modal-header-content {
  padding-left: 1.25rem;
}

.pug-create-form {
  padding: 1.25rem;
  gap: 0.85rem;
}

.pug-create-teams-row {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: end;
  gap: 0.6rem;
}

.pug-create-vs {
  font-size: 0.72rem;
  font-weight: 800;
  letter-spacing: 0.06em;
  color: var(--ink-3);
  padding-bottom: 0.65rem;
  text-align: center;
}

.pug-create-footer-section {
  padding-top: 0.25rem;
}

/* ── Stats bar ───────────────────────────────────────────────────────────── */
.pug-stats-bar {
  display: flex;
  gap: 0;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.pug-stat-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.18rem;
  padding: 0.85rem 1.1rem;
  border-right: 1px solid var(--line);
}

.pug-stat-item:last-child {
  border-right: none;
}

.pug-stat-value {
  font-size: 1.35rem;
  font-weight: 800;
  color: var(--ink-1);
  line-height: 1;
  display: flex;
  align-items: baseline;
  gap: 0.25rem;
}

.pug-stat-of {
  font-size: 0.85rem;
  font-weight: 500;
}

.pug-stat-label {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--ink-2);
}

/* ── Match grid ──────────────────────────────────────────────────────────── */
.pug-match-grid {
  list-style: none;
  padding: 0;
  margin: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 0.75rem;
}

.pug-empty {
  color: var(--ink-3);
  font-size: 0.95rem;
  text-align: center;
  padding: 2rem 1rem;
}

/* ── Match card ──────────────────────────────────────────────────────────── */
.pug-match-card {
  position: relative;
  display: flex;
  background: var(--bg-1);
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  overflow: hidden;
  cursor: pointer;
  outline: none;
  transition: box-shadow 0.15s, border-color 0.15s;
}

.pug-match-card:hover,
.pug-match-card:focus-visible {
  box-shadow: 0 8px 24px rgba(16, 34, 72, 0.14), 0 2px 6px rgba(16, 34, 72, 0.1);
  border-color: color-mix(in srgb, var(--brand-2) 40%, var(--line) 60%);
}

.pug-match-card:focus-visible {
  box-shadow: 0 0 0 2px var(--brand-1), 0 8px 24px rgba(16, 34, 72, 0.14);
}

.match-card-stripe {
  width: 5px;
  flex-shrink: 0;
  background: linear-gradient(180deg, var(--stripe-a), var(--stripe-b));
}

.match-card-body {
  flex: 1;
  min-width: 0;
  padding: 0.85rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.match-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.match-card-title {
  font-size: 0.78rem;
  font-weight: 700;
  color: var(--ink-2);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.match-card-badge {
  flex-shrink: 0;
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  padding: 0.18rem 0.55rem;
  border-radius: var(--radius-pill);
  border: 1px solid var(--line);
  color: var(--ink-3);
  background: color-mix(in srgb, var(--card) 85%, #1a2740 15%);
}

.badge-ready {
  color: var(--brand-1);
  background: color-mix(in srgb, var(--brand-1) 12%, var(--card) 88%);
  border-color: color-mix(in srgb, var(--brand-1) 44%, var(--line) 56%);
}

.badge-done {
  color: #25b177;
  background: color-mix(in srgb, #1d8e61 16%, var(--card) 84%);
  border-color: color-mix(in srgb, #25b177 50%, var(--line) 50%);
}

.match-card-matchup {
  display: flex;
  flex-direction: column;
  gap: 0;
  margin: 0.3rem 0;
}

.match-card-team-row {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  font-size: 0.88rem;
  font-weight: 700;
  color: var(--ink-1);
  padding: 0.3rem 0.6rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--bg-1) 60%, #0d1526 40%);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
}

.match-card-team-row.is-winner {
  color: #25b177;
  background: color-mix(in srgb, #1d8e61 18%, var(--bg-1) 82%);
  border-color: color-mix(in srgb, #25b177 45%, var(--line) 55%);
}

.match-team-trophy {
  font-size: 0.88rem;
  flex-shrink: 0;
  color: #f7b801;
}

.match-card-vs-divider {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 0.2rem 0;
}

.match-card-vs-divider::before,
.match-card-vs-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--line);
}

.vs-badge {
  padding: 0.1rem 0.42rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-sm);
  font-size: 0.66rem;
  font-weight: 800;
  letter-spacing: 0.06em;
  color: var(--ink-3);
  background: var(--card);
  margin: 0 0.35rem;
  line-height: 1.4;
}

.match-card-footer {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  border-top: 1px solid var(--line);
  padding-top: 0.55rem;
  margin-top: auto;
}

.match-card-footer-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.match-card-date-row {
  justify-content: flex-start;
}

.match-card-meta {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.82rem;
  color: var(--ink-3);
}

.match-card-meta .material-symbols-rounded {
  font-size: 0.95rem;
}

/* ── Modal overlay ───────────────────────────────────────────────────────── */
.pug-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(8, 16, 36, 0.72);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 900;
  padding: 1rem;
}

.pug-modal {
  width: 100%;
  max-width: 580px;
  max-height: 90vh;
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ── Modal header (fixed, not scrolled) ──────────────────────────────────── */
.pug-modal-header {
  display: flex;
  align-items: flex-start;
  gap: 0;
  border-bottom: 1px solid var(--line);
  flex-shrink: 0;
}

.modal-header-stripe {
  width: 5px;
  align-self: stretch;
  flex-shrink: 0;
  background: linear-gradient(180deg, var(--stripe-a), var(--stripe-b));
}

.modal-header-content {
  flex: 1;
  min-width: 0;
  padding: 1rem 0.75rem 1rem 1rem;
}

.modal-title-row {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  flex-wrap: wrap;
}

.modal-title {
  font-size: 1.15rem;
  font-weight: 800;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin: 0;
}

.modal-subtitle {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.45rem;
  font-size: 0.86rem;
  color: var(--ink-3);
  margin: 0.35rem 0 0;
}

.modal-meta-item {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
}

.modal-meta-icon {
  font-size: 0.95rem;
}

.modal-meta-sep {
  color: var(--ink-3);
  line-height: 1;
}

.modal-close-btn {
  flex-shrink: 0;
  margin: 0.65rem 0.65rem 0 0;
}

/* ── Modal scrollable body ───────────────────────────────────────────────── */
.pug-modal-body {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

/* ── Modal sections ──────────────────────────────────────────────────────── */
.modal-section {
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--line);
}

.modal-section-title {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--ink-3);
  margin: 0 0 0.7rem;
}

.modal-hint-text {
  font-size: 0.9rem;
  color: var(--ink-3);
  margin: 0;
}

/* ── Schedule editor ─────────────────────────────────────────────────────── */
.form-label-hint {
  font-size: 0.8rem;
  font-weight: 400;
  color: var(--ink-3);
}

.schedule-editor-row {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  flex-wrap: wrap;
}

.schedule-date-input {
  flex: 1;
  min-width: 200px;
}

/* ── Matchup editor ──────────────────────────────────────────────────────── */
.matchup-editor-row {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  flex-wrap: wrap;
}

.vs-sep {
  font-size: 0.8rem;
  color: var(--ink-3);
  font-weight: 600;
}

.matchup-editor-row select {
  flex: 1;
  min-width: 130px;
}

.matchup-hint {
  margin: 0.4rem 0 0;
  font-size: 0.85rem;
  font-weight: 600;
}

.matchup-hint.is-error {
  color: #cf5f4a;
}

.matchup-display-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  flex-wrap: wrap;
}

/* ── Winner section ──────────────────────────────────────────────────────── */
.winner-declare-row {
  display: flex;
  gap: 0.65rem;
  flex-wrap: wrap;
}

.winner-declare-btn {
  flex: 1;
  min-width: 140px;
  justify-content: center;
}

.winner-result-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  flex-wrap: wrap;
}

.winner-result-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1rem;
}

.winner-trophy {
  color: #f7b801;
  font-size: 1.25rem;
}

/* ── Player lists ────────────────────────────────────────────────────────── */
.modal-roster-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.25rem;
}

.modal-team-col {
  min-width: 0;
}

.modal-team-name {
  font-size: 0.78rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--ink-2);
  margin: 0 0 0.55rem;
}

.modal-team-name--unassigned {
  margin-top: 1rem;
}

.modal-player-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.modal-player-row {
  padding: 0.45rem 0.65rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
}

/* ── Danger zone ─────────────────────────────────────────────────────────── */
.modal-danger-zone {
  padding: 1rem 1.25rem;
  margin-top: auto;
}

/* ── Mobile ──────────────────────────────────────────────────────────────── */
@media (max-width: 900px) {
  .pug-match-grid {
    grid-template-columns: 1fr;
  }

  .pug-stats-bar {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
  }

  .pug-stat-item:nth-child(2) {
    border-right: none;
  }

  .pug-stat-item:nth-child(3) {
    border-top: 1px solid var(--line);
    border-right: 1px solid var(--line);
  }

  .pug-stat-item:nth-child(4) {
    border-top: 1px solid var(--line);
  }

  .modal-roster-grid {
    grid-template-columns: 1fr;
  }

  .matchup-editor-row {
    flex-direction: column;
    align-items: stretch;
  }

  .matchup-editor-row .vs-sep {
    text-align: center;
  }

  .winner-declare-row {
    flex-direction: column;
  }

  .winner-declare-btn {
    min-width: unset;
  }

  .pug-modal-overlay {
    align-items: flex-end;
    padding: 0;
  }

  .pug-modal {
    max-width: 100%;
    max-height: 88vh;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  }
}
</style>
