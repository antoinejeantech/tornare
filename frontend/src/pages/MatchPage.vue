<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { averagePlayersElo, formatAverageElo } from '../lib/elo'
import { sortPlayersByRoleThenName } from '../lib/roles'
import { useConfirm } from '../lib/confirm'
import { useMatchStore } from '../stores/match'
import PlayerIdentity from '../components/player/PlayerIdentity.vue'

const route = useRoute()
const router = useRouter()
const confirm = useConfirm()
const matchStore = useMatchStore()

const match = ref(null)
const error = ref('')
const notice = ref('')
const loadingMatch = ref(false)
const deletingMatch = ref(false)

const matchId = computed(() => String(route.params.id || ''))
const isEventMatch = computed(() => Boolean(match.value?.event_id))
const hasEventMatchup = computed(() => Boolean(match.value?.team_a_id && match.value?.team_b_id))

const teamAPlayers = computed(() => {
  if (!match.value?.team_a_id) {
    return []
  }

  return sortPlayersByRoleThenName(
    match.value.players.filter((player) => player.team_id === match.value.team_a_id)
  )
})

const teamBPlayers = computed(() => {
  if (!match.value?.team_b_id) {
    return []
  }

  return sortPlayersByRoleThenName(
    match.value.players.filter((player) => player.team_id === match.value.team_b_id)
  )
})

const teamAAverageElo = computed(() => averagePlayersElo(teamAPlayers.value))
const teamBAverageElo = computed(() => averagePlayersElo(teamBPlayers.value))

function setError(message) {
  error.value = message
  notice.value = ''
}

function clearError() {
  error.value = ''
}

function setNotice(message) {
  notice.value = message
}

function clearNotice() {
  notice.value = ''
}

async function loadMatch() {
  if (!matchId.value) {
    match.value = null
    return
  }

  loadingMatch.value = true
  try {
    clearError()
    clearNotice()
    match.value = await matchStore.fetchMatch(matchId.value)
  } catch (err) {
    match.value = null
    setError(err instanceof Error ? err.message : 'Failed to load match')
  } finally {
    loadingMatch.value = false
  }
}

async function deleteMatch() {
  if (!match.value || deletingMatch.value) {
    return
  }

  const confirmed = await confirm.ask({
    title: 'Delete match?',
    message: `Delete match "${match.value.title}"?`,
    confirmText: 'Delete match',
    tone: 'danger',
  })
  if (!confirmed) {
    return
  }

  deletingMatch.value = true
  try {
    clearError()
    clearNotice()

    await matchStore.deleteMatch(matchId.value)

    router.push({ name: 'home' })
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to delete match')
  } finally {
    deletingMatch.value = false
  }
}

function navigateToHome() {
  if (match.value?.event_id) {
    router.push({ name: 'event', params: { id: match.value.event_id } })
    return
  }

  router.push({ name: 'home' })
}

watch(
  () => route.params.id,
  () => {
    loadMatch()
  }
)

onMounted(loadMatch)
</script>

<template>
  <main class="app-shell">
    <header class="page-header">
      <h1 class="page-title">Match</h1>
    </header>

    <p v-if="error" class="status status-error">{{ error }}</p>
    <p v-else-if="notice" class="status status-ok">{{ notice }}</p>

    <section v-if="loadingMatch" class="card">
      <p>Loading match...</p>
    </section>

    <section v-else-if="match" class="card">
      <h2>Match: {{ match.title }}</h2>
      <p class="muted">{{ match.players.length }}/{{ match.max_players }} players · {{ match.map }}</p>

      <p v-if="isEventMatch" class="muted">
        This match uses event teams. Manage roster and team assignment from the event page.
      </p>
      <p v-else class="muted">
        Non-event match players are no longer supported.
      </p>

      <h3>Team management</h3>
      <template v-if="isEventMatch">
        <p v-if="!hasEventMatchup" class="muted">
          Set both teams from the event page matchup editor to view opposing rosters here.
        </p>
        <div v-else class="matchup-rosters">
          <section class="team-roster-card">
            <h4 class="team-roster-title">{{ match.team_a_name || 'Team 1' }}</h4>
            <p class="muted team-avg-elo">{{ formatAverageElo(teamAAverageElo) }}</p>
            <p v-if="teamAPlayers.length === 0" class="muted">No players assigned to this team.</p>
            <ul v-else class="match-players-list">
              <li v-for="player in teamAPlayers" :key="`a-${player.id}`" class="match-player-row">
                <PlayerIdentity :name="player.name" :role="player.role" :rank="player.rank" />
              </li>
            </ul>
          </section>

          <section class="team-roster-card">
            <h4 class="team-roster-title">{{ match.team_b_name || 'Team 2' }}</h4>
            <p class="muted team-avg-elo">{{ formatAverageElo(teamBAverageElo) }}</p>
            <p v-if="teamBPlayers.length === 0" class="muted">No players assigned to this team.</p>
            <ul v-else class="match-players-list">
              <li v-for="player in teamBPlayers" :key="`b-${player.id}`" class="match-player-row">
                <PlayerIdentity :name="player.name" :role="player.role" :rank="player.rank" />
              </li>
            </ul>
          </section>
        </div>
      </template>

      <div class="match-edit-actions">
        <button
          class="btn-danger icon-btn"
          :disabled="deletingMatch"
          :title="deletingMatch ? 'Deleting match' : 'Delete match'"
          @click="deleteMatch"
        >
          <span class="material-symbols-rounded" aria-hidden="true">
            {{ deletingMatch ? 'hourglass_top' : 'delete_forever' }}
          </span>
          <span class="sr-only">{{ deletingMatch ? 'Deleting match' : 'Delete match' }}</span>
        </button>
      </div>
    </section>

    <section v-else class="card">
      <h2>Match not found</h2>
      <p class="muted">This match may have been deleted.</p>
      <button class="btn-secondary" @click="navigateToHome">Back to matches</button>
    </section>
  </main>
</template>

<style scoped>
.match-players-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.match-player-row {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-1) 8%);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
  border-radius: var(--radius-item);
  padding: 0.64rem 0.7rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.65rem;
}

.match-edit-actions {
  margin-top: 0.85rem;
}

.matchup-rosters {
  display: grid;
  gap: 0.75rem;
}

.team-roster-card {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-2) 10%);
  border-radius: var(--radius-item);
  background: color-mix(in srgb, var(--card) 92%, #19253a 8%);
  padding: 0.6rem;
}

.team-roster-title {
  margin: 0 0 0.45rem;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.team-avg-elo {
  margin: -0.2rem 0 0.5rem;
}

@media (min-width: 860px) {
  .matchup-rosters {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
