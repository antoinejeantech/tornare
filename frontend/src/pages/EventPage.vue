<script setup>
import { computed, onMounted, provide, proxyRefs, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getRankIcon, overwatchRanks } from '../lib/ranks'
import { useAlert } from '../lib/alerts'
import { useEventStore } from '../stores/event'
import { useMatchStore } from '../stores/match'
import RosterSection from '../components/event/RosterSection.vue'
import TeamsSection from '../components/event/TeamsSection.vue'
import MatchesSection from '../components/event/MatchesSection.vue'
import OverviewSection from '../components/event/OverviewSection.vue'
import SignupRequestsSection from '../components/event/SignupRequestsSection.vue'

const route = useRoute()
const router = useRouter()
const alert = useAlert()
const eventStore = useEventStore()
const matchStore = useMatchStore()

const event = ref(null)
const loadingEvent = ref(false)
const updatingEvent = ref(false)
const editingEventMeta = ref(false)
const creatingMatch = ref(false)
const deletingEvent = ref(false)
const deletingMatchId = ref(null)
const addingPlayer = ref(false)
const deletingPlayers = ref({})
const creatingTeam = ref(false)
const deletingTeams = ref({})
const savingPlayerTeams = ref({})
const savingPlayerEdits = ref({})
const savingTeamEdits = ref({})
const savingMatchups = ref({})
const loadingSignupRequests = ref(false)
const signupRequests = ref([])
const reviewingSignupRequests = ref({})
const signupToken = ref('')

const newMatchTitle = ref('')
const newMatchMap = ref('')
const newPlayerName = ref('')
const newPlayerRole = ref('DPS')
const newPlayerRank = ref('Unranked')
const newTeamName = ref('')
const editingPlayerId = ref(null)
const editPlayerName = ref('')
const editPlayerRole = ref('DPS')
const editPlayerRank = ref('Unranked')
const editingTeamId = ref(null)
const editTeamName = ref('')
const teamAssignmentSelections = ref({})
const matchupSelections = ref({})
const editEventName = ref('')
const editEventMaxPlayers = ref(10)
const activeSection = ref('overview')

const eventId = computed(() => String(route.params.id || ''))
const canManageEvent = computed(() => Boolean(event.value?.is_owner))
const signupShareUrl = computed(() => {
  if (!signupToken.value) {
    return ''
  }

  if (typeof window === 'undefined') {
    return `/join/${signupToken.value}`
  }

  return `${window.location.origin}/join/${signupToken.value}`
})

const canCreateMatch = computed(() => {
  return (
    Boolean(event.value) &&
    newMatchTitle.value.trim().length > 0 &&
    newMatchMap.value.trim().length > 0
  )
})

const canAddPlayer = computed(() => {
  return Boolean(event.value) && newPlayerName.value.trim().length > 0
})

const canCreateTeam = computed(() => {
  return Boolean(event.value) && newTeamName.value.trim().length > 0
})

const canSaveEventMeta = computed(() => {
  const nameOk = editEventName.value.trim().length > 0
  const maxOk = Number.isInteger(editEventMaxPlayers.value) && editEventMaxPlayers.value >= 2 && editEventMaxPlayers.value <= 12
  return nameOk && maxOk
})

const eventIsFull = computed(() => {
  if (!event.value) {
    return false
  }
  return event.value.players.length >= event.value.max_players
})

function setError(message) {
  alert.error(message)
}

function setNotice(message) {
  alert.success(message)
}

function resetFeedback() {
  // Global alerts are transient and don't require manual reset.
}

function ensureOwnerAction() {
  if (canManageEvent.value) {
    return true
  }

  setError('Only the event owner can modify this event.')
  return false
}

function hydrateSelections() {
  if (!event.value) {
    teamAssignmentSelections.value = {}
    matchupSelections.value = {}
    return
  }

  const nextTeamAssignments = {}
  for (const team of event.value.teams) {
    nextTeamAssignments[team.id] = ''
  }

  const nextMatchups = {}
  for (const match of event.value.matches) {
    nextMatchups[match.id] = {
      teamAId: match.team_a_id ? String(match.team_a_id) : '',
      teamBId: match.team_b_id ? String(match.team_b_id) : ''
    }
  }

  teamAssignmentSelections.value = nextTeamAssignments
  matchupSelections.value = nextMatchups
}

async function loadEvent() {
  if (!eventId.value) {
    event.value = null
    return
  }

  loadingEvent.value = true
  try {
    resetFeedback()
    event.value = await eventStore.fetchEvent(eventId.value)
    hydrateSelections()
    if (event.value?.is_owner) {
      await loadOwnerSignupData()
    } else {
      signupToken.value = ''
      signupRequests.value = []
    }
  } catch (err) {
    event.value = null
    setError(err instanceof Error ? err.message : 'Failed to load event')
  } finally {
    loadingEvent.value = false
  }
}

async function loadOwnerSignupData() {
  if (!eventId.value || !canManageEvent.value) {
    return
  }

  loadingSignupRequests.value = true
  try {
    const [linkResponse, requests] = await Promise.all([
      eventStore.fetchSignupLink(eventId.value),
      eventStore.listSignupRequests(eventId.value),
    ])

    signupToken.value = linkResponse.signup_token || ''
    signupRequests.value = Array.isArray(requests) ? requests : []
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to load signup requests')
  } finally {
    loadingSignupRequests.value = false
  }
}

async function copySignupLink() {
  if (!signupShareUrl.value) {
    return
  }

  try {
    await navigator.clipboard.writeText(signupShareUrl.value)
    setNotice('Signup link copied')
  } catch {
    setError('Could not copy signup link')
  }
}

async function acceptSignupRequest(requestId) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || reviewingSignupRequests.value[requestId]) {
    return
  }

  reviewingSignupRequests.value = {
    ...reviewingSignupRequests.value,
    [requestId]: true,
  }

  try {
    const updatedEvent = await eventStore.acceptSignupRequest(eventId.value, requestId)
    event.value = updatedEvent
    hydrateSelections()
    await loadOwnerSignupData()
    setNotice('Signup request accepted')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to accept signup request')
  } finally {
    reviewingSignupRequests.value = {
      ...reviewingSignupRequests.value,
      [requestId]: false,
    }
  }
}

async function declineSignupRequest(requestId) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || reviewingSignupRequests.value[requestId]) {
    return
  }

  reviewingSignupRequests.value = {
    ...reviewingSignupRequests.value,
    [requestId]: true,
  }

  try {
    await eventStore.declineSignupRequest(eventId.value, requestId)
    await loadOwnerSignupData()
    setNotice('Signup request declined')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to decline signup request')
  } finally {
    reviewingSignupRequests.value = {
      ...reviewingSignupRequests.value,
      [requestId]: false,
    }
  }
}

async function createTeam() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || !canCreateTeam.value || creatingTeam.value) {
    return
  }

  creatingTeam.value = true
  try {
    resetFeedback()

    const updatedEvent = await eventStore.createTeam(eventId.value, newTeamName.value.trim())

    event.value = updatedEvent
    hydrateSelections()
    newTeamName.value = ''
    setNotice('Team created')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to create team')
  } finally {
    creatingTeam.value = false
  }
}

async function saveTeamEdit(teamId) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || !editTeamName.value.trim() || savingTeamEdits.value[teamId]) {
    return
  }

  savingTeamEdits.value = {
    ...savingTeamEdits.value,
    [teamId]: true
  }

  try {
    resetFeedback()

    const updatedEvent = await eventStore.updateTeam(eventId.value, teamId, editTeamName.value.trim())

    event.value = updatedEvent
    hydrateSelections()
    editingTeamId.value = null
    editTeamName.value = ''
    setNotice('Team updated')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to update team')
  } finally {
    savingTeamEdits.value = {
      ...savingTeamEdits.value,
      [teamId]: false
    }
  }
}

async function deleteTeam(team) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || deletingTeams.value[team.id]) {
    return
  }

  const confirmed = window.confirm(`Delete team "${team.name}"?`)
  if (!confirmed) {
    return
  }

  deletingTeams.value = {
    ...deletingTeams.value,
    [team.id]: true
  }

  try {
    resetFeedback()

    await eventStore.deleteTeam(eventId.value, team.id)

    await loadEvent()
    setNotice('Team deleted')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to delete team')
  } finally {
    deletingTeams.value = {
      ...deletingTeams.value,
      [team.id]: false
    }
  }
}

async function addPlayer() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || !canAddPlayer.value || addingPlayer.value) {
    return
  }

  if (eventIsFull.value) {
    setError('This event roster is full. Increase max players or remove a player.')
    return
  }

  addingPlayer.value = true
  try {
    resetFeedback()

    const updatedEvent = await eventStore.addPlayer(eventId.value, {
      name: newPlayerName.value.trim(),
      role: newPlayerRole.value,
      rank: newPlayerRank.value,
    })

    event.value = updatedEvent
    hydrateSelections()

    newPlayerName.value = ''
    newPlayerRole.value = 'DPS'
    newPlayerRank.value = 'Unranked'
    setNotice('Player added to event roster')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to add player')
  } finally {
    addingPlayer.value = false
  }
}

async function savePlayerEdit(playerId) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || !editPlayerName.value.trim() || savingPlayerEdits.value[playerId]) {
    return
  }

  savingPlayerEdits.value = {
    ...savingPlayerEdits.value,
    [playerId]: true
  }

  try {
    resetFeedback()

    const updatedEvent = await eventStore.updatePlayer(eventId.value, playerId, {
      name: editPlayerName.value.trim(),
      role: editPlayerRole.value,
      rank: editPlayerRank.value,
    })

    event.value = updatedEvent
    hydrateSelections()
    editingPlayerId.value = null
    editPlayerName.value = ''
    editPlayerRole.value = 'DPS'
    editPlayerRank.value = 'Unranked'
    setNotice('Player updated')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to update player')
  } finally {
    savingPlayerEdits.value = {
      ...savingPlayerEdits.value,
      [playerId]: false
    }
  }
}

async function setPlayerTeam(playerId, teamId) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || savingPlayerTeams.value[playerId]) {
    return
  }

  savingPlayerTeams.value = {
    ...savingPlayerTeams.value,
    [playerId]: true
  }

  try {
    resetFeedback()

    const updatedEvent = await eventStore.assignPlayerTeam(eventId.value, playerId, teamId)

    event.value = updatedEvent
    hydrateSelections()
    setNotice('Team assignment saved')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to assign team')
  } finally {
    savingPlayerTeams.value = {
      ...savingPlayerTeams.value,
      [playerId]: false
    }
  }
}

async function assignSelectedPlayerToTeam(teamId) {
  const selected = teamAssignmentSelections.value[teamId]
  if (!selected) {
    return
  }

  await setPlayerTeam(selected, teamId)
  teamAssignmentSelections.value = {
    ...teamAssignmentSelections.value,
    [teamId]: ''
  }
}

async function removePlayerFromTeam(playerId) {
  await setPlayerTeam(playerId, null)
}

async function removePlayer(player) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || deletingPlayers.value[player.id]) {
    return
  }

  const confirmed = window.confirm(`Remove player "${player.name}" from this event?`)
  if (!confirmed) {
    return
  }

  deletingPlayers.value = {
    ...deletingPlayers.value,
    [player.id]: true
  }

  const previousEvent = event.value
  const previousTeamSelections = { ...teamAssignmentSelections.value }

  if (event.value) {
    event.value = {
      ...event.value,
      players: event.value.players.filter((current) => current.id !== player.id),
      matches: event.value.matches.map((currentMatch) => ({
        ...currentMatch,
        players: currentMatch.players.filter((currentPlayer) => currentPlayer.id !== player.id)
      }))
    }
  }

  try {
    resetFeedback()

    await eventStore.deletePlayer(eventId.value, player.id)

    setNotice('Player removed from event roster')
  } catch (err) {
    event.value = previousEvent
    teamAssignmentSelections.value = previousTeamSelections
    setError(err instanceof Error ? err.message : 'Failed to remove player')
    await loadEvent()
  } finally {
    deletingPlayers.value = {
      ...deletingPlayers.value,
      [player.id]: false
    }
  }
}

async function saveMatchup(matchId) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || savingMatchups.value[matchId]) {
    return
  }

  const selection = matchupSelections.value[matchId] || { teamAId: '', teamBId: '' }
  const teamAId = selection.teamAId || null
  const teamBId = selection.teamBId || null

  savingMatchups.value = {
    ...savingMatchups.value,
    [matchId]: true
  }

  try {
    resetFeedback()

    const updatedMatch = await matchStore.setMatchupForEvent(eventId.value, matchId, {
      team_a_id: teamAId,
      team_b_id: teamBId,
    })

    if (event.value) {
      event.value = {
        ...event.value,
        matches: event.value.matches.map((match) => (match.id === matchId ? updatedMatch : match))
      }
      hydrateSelections()
    }

    setNotice('Matchup saved')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to save matchup')
    await loadEvent()
  } finally {
    savingMatchups.value = {
      ...savingMatchups.value,
      [matchId]: false
    }
  }
}

async function createMatch() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || !canCreateMatch.value || creatingMatch.value) {
    return
  }

  creatingMatch.value = true
  try {
    resetFeedback()

    const created = await matchStore.createMatchForEvent(eventId.value, {
      title: newMatchTitle.value.trim(),
      map: newMatchMap.value.trim(),
    })

    if (event.value) {
      event.value = {
        ...event.value,
        matches: [created, ...event.value.matches]
      }

      matchupSelections.value = {
        ...matchupSelections.value,
        [created.id]: {
          teamAId: created.team_a_id ? String(created.team_a_id) : '',
          teamBId: created.team_b_id ? String(created.team_b_id) : ''
        }
      }
    }

    newMatchTitle.value = ''
    newMatchMap.value = ''
    setNotice('Match created in event')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to create match')
  } finally {
    creatingMatch.value = false
  }
}

async function deleteMatch(matchId) {
  if (!ensureOwnerAction()) {
    return
  }

  if (deletingMatchId.value) {
    return
  }

  const target = event.value?.matches.find((match) => match.id === matchId)
  const confirmed = window.confirm(`Delete match "${target?.title || matchId}"?`)
  if (!confirmed) {
    return
  }

  deletingMatchId.value = matchId
  try {
    resetFeedback()

    await matchStore.deleteMatch(matchId)

    if (event.value) {
      event.value = {
        ...event.value,
        matches: event.value.matches.filter((match) => match.id !== matchId)
      }
    }

    setNotice('Match deleted')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to delete match')
  } finally {
    deletingMatchId.value = null
  }
}

async function deleteEvent() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!event.value || deletingEvent.value) {
    return
  }

  const confirmed = window.confirm(`Delete event "${event.value.name}" and all its matches?`)
  if (!confirmed) {
    return
  }

  deletingEvent.value = true
  try {
    resetFeedback()

    await eventStore.deleteEvent(eventId.value)

    router.push({ name: 'home' })
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to delete event')
  } finally {
    deletingEvent.value = false
  }
}

function startEditEvent() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!event.value) {
    return
  }

  editEventName.value = event.value.name
  editEventMaxPlayers.value = Number(event.value.max_players)
  editingEventMeta.value = true
}

function cancelEditEvent() {
  editingEventMeta.value = false
}

async function saveEventEdit() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!event.value || updatingEvent.value || !canSaveEventMeta.value) {
    return
  }

  updatingEvent.value = true
  try {
    resetFeedback()

    const payloadType = String(event.value.event_type).trim().toUpperCase() === 'TOURNEY' ? 'TOURNEY' : 'PUG'

    const updatedEvent = await eventStore.updateEvent(eventId.value, {
      name: editEventName.value.trim(),
      event_type: payloadType,
      max_players: editEventMaxPlayers.value,
    })

    event.value = updatedEvent
    hydrateSelections()
    editingEventMeta.value = false
    setNotice('Event updated')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to update event')
  } finally {
    updatingEvent.value = false
  }
}

function navigateToHome() {
  router.push({ name: 'home' })
}

function openSection(section) {
  activeSection.value = section
}

watch(
  () => route.params.id,
  () => {
    loadEvent()
  }
)

onMounted(loadEvent)

provide('eventCtx', proxyRefs({
  event,
  eventIsFull,
  loadingEvent,
  creatingTeam,
  creatingMatch,
  deletingEvent,
  deletingMatchId,
  addingPlayer,
  deletingPlayers,
  deletingTeams,
  savingPlayerTeams,
  savingPlayerEdits,
  savingTeamEdits,
  savingMatchups,
  newTeamName,
  newMatchTitle,
  newMatchMap,
  newPlayerName,
  newPlayerRole,
  newPlayerRank,
  editTeamName,
  editingTeamId,
  editPlayerName,
  editPlayerRole,
  editPlayerRank,
  editingPlayerId,
  teamAssignmentSelections,
  matchupSelections,
  canCreateTeam,
  canCreateMatch,
  canAddPlayer,
  canManageEvent,
  signupRequests,
  loadingSignupRequests,
  reviewingSignupRequests,
  signupShareUrl,
  openSection,
  createTeam,
  createMatch,
  deleteEvent,
  deleteMatch,
  saveMatchup,
  saveTeamEdit,
  deleteTeam,
  assignSelectedPlayerToTeam,
  removePlayerFromTeam,
  savePlayerEdit,
  addPlayer,
  removePlayer,
  copySignupLink,
  acceptSignupRequest,
  declineSignupRequest,
  getRankIcon,
  overwatchRanks,
}))
</script>

<template>
  <main class="app-shell event-shell">
    <header class="page-header">
      <h1 class="page-title">Event Setup</h1>
    </header>

    <section v-if="loadingEvent" class="card">
      <p>Loading event...</p>
    </section>

    <section v-else-if="event" class="card event-workspace-card">
      <div class="event-header-row">
        <h2>{{ event.name }}</h2>
        <p v-if="!canManageEvent" class="muted owner-note">Read-only view. Only the event owner can edit this event.</p>
        <div class="event-header-actions">
          <button
            v-if="canManageEvent && !editingEventMeta"
            class="btn-secondary icon-btn"
            :disabled="updatingEvent"
            :title="updatingEvent ? 'Saving event' : 'Edit event details'"
            @click="startEditEvent"
          >
            <span class="material-symbols-rounded" aria-hidden="true">
              {{ updatingEvent ? 'hourglass_top' : 'edit' }}
            </span>
            <span class="sr-only">{{ updatingEvent ? 'Saving event' : 'Edit event details' }}</span>
          </button>
          <button
            v-if="canManageEvent && editingEventMeta"
            class="btn-primary icon-btn"
            :disabled="updatingEvent || !canSaveEventMeta"
            :title="updatingEvent ? 'Saving event' : 'Save event'"
            @click="saveEventEdit"
          >
            <span class="material-symbols-rounded" aria-hidden="true">
              {{ updatingEvent ? 'hourglass_top' : 'save' }}
            </span>
            <span class="sr-only">{{ updatingEvent ? 'Saving event' : 'Save event' }}</span>
          </button>
          <button
            v-if="canManageEvent && editingEventMeta"
            class="btn-secondary icon-btn"
            :disabled="updatingEvent"
            title="Cancel event edit"
            @click="cancelEditEvent"
          >
            <span class="material-symbols-rounded" aria-hidden="true">close</span>
            <span class="sr-only">Cancel event edit</span>
          </button>
          <button
            v-if="canManageEvent && !editingEventMeta"
            class="btn-danger icon-btn"
            :disabled="deletingEvent"
            :title="deletingEvent ? 'Deleting event' : 'Delete event'"
            @click="deleteEvent"
          >
            <span class="material-symbols-rounded" aria-hidden="true">
              {{ deletingEvent ? 'hourglass_top' : 'delete_forever' }}
            </span>
            <span class="sr-only">{{ deletingEvent ? 'Deleting event' : 'Delete event' }}</span>
          </button>
        </div>
      </div>
      <form v-if="editingEventMeta" class="event-edit-form" @submit.prevent="saveEventEdit">
        <label>
          Event name
          <input v-model="editEventName" placeholder="Event name" />
        </label>
        <label>
          Max players
          <input v-model.number="editEventMaxPlayers" type="number" min="2" max="12" step="1" />
        </label>
      </form>
      <div class="event-meta-row">
        <span class="meta-chip">{{ event.event_type }}</span>
        <span class="meta-chip">by {{ event.creator_name || 'Unknown' }}</span>
        <span class="meta-chip">{{ event.players.length }}/{{ event.max_players }} players</span>
        <span class="meta-chip">{{ event.teams.length }} teams</span>
        <span class="meta-chip">{{ event.matches.length }} matches</span>
      </div>

      <div class="event-layout">
        <aside class="event-left-nav" aria-label="Event sections">
          <button class="left-nav-item" :class="{ active: activeSection === 'overview' }" @click="activeSection = 'overview'">Overview</button>
          <button class="left-nav-item" :class="{ active: activeSection === 'roster' }" @click="activeSection = 'roster'">Roster</button>
          <button class="left-nav-item" :class="{ active: activeSection === 'teams' }" @click="activeSection = 'teams'">Teams</button>
          <button class="left-nav-item" :class="{ active: activeSection === 'matches' }" @click="activeSection = 'matches'">Matches</button>
          <button v-if="canManageEvent" class="left-nav-item" :class="{ active: activeSection === 'requests' }" @click="activeSection = 'requests'">Requests</button>
        </aside>

        <section class="event-panel">
          <OverviewSection v-if="activeSection === 'overview'" />
          <RosterSection v-else-if="activeSection === 'roster'" />
          <TeamsSection v-else-if="activeSection === 'teams'" />
          <MatchesSection v-else-if="activeSection === 'matches'" />
          <SignupRequestsSection v-else-if="activeSection === 'requests' && canManageEvent" />
          <OverviewSection v-else />
        </section>
      </div>

    </section>

    <section v-else class="card">
      <h2>Event not found</h2>
      <p class="muted">This event may have been deleted.</p>
      <button class="btn-secondary" @click="navigateToHome">Back to events</button>
    </section>
  </main>
</template>

<style scoped>
.event-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.event-shell {
  max-width: none;
  width: 100%;
  padding: 1.1rem 1.2rem 1.25rem;
}

.event-workspace-card {
  min-height: calc(100vh - 200px);
  display: flex;
  flex-direction: column;
}

.event-header-row h2 {
  margin: 0;
}

.event-header-actions {
  display: flex;
  gap: 0.35rem;
}

.event-edit-form {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 0.7fr);
  gap: 0.5rem;
  margin: 0.55rem 0 0.7rem;
}

.event-edit-form label {
  display: grid;
  gap: 0.24rem;
}

.event-meta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.48rem;
  margin-bottom: 0.75rem;
}

.meta-chip {
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--brand-1) 35%, var(--line) 65%);
  background: color-mix(in srgb, var(--accent) 22%, var(--meta-bg) 78%);
  color: var(--meta-ink);
  padding: 0.22rem 0.62rem;
  font-size: 0.81rem;
  font-family: "Space Mono", ui-monospace, monospace;
  font-weight: 700;
  text-transform: uppercase;
}

.event-grid {
  display: grid;
  gap: 0.72rem;
  grid-template-columns: 1fr;
  margin-bottom: 0.8rem;
}

.event-layout {
  display: grid;
  grid-template-columns: 200px minmax(0, 1fr);
  gap: 0.75rem;
  align-items: stretch;
  margin-bottom: 0;
  flex: 1;
  min-height: 0;
}

.event-left-nav {
  display: grid;
  gap: 0.34rem;
  border: 1px solid color-mix(in srgb, var(--brand-1) 24%, var(--line) 76%);
  border-radius: 14px;
  padding: 0.48rem;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--card) 88%, #edf4ff 12%), color-mix(in srgb, var(--card) 96%, #f4f8ff 4%));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.06),
    0 6px 16px rgba(16, 39, 82, 0.12);
  align-self: start;
  height: fit-content;
}

.left-nav-item {
  width: 100%;
  text-align: left;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  background: color-mix(in srgb, var(--card) 92%, #f4f8ff 8%);
  color: var(--ink-2);
  border-radius: 10px;
  padding: 0.55rem 0.62rem;
  font-weight: 760;
  letter-spacing: 0.01em;
  cursor: pointer;
  transition:
    background 0.16s ease,
    border-color 0.16s ease,
    color 0.16s ease,
    transform 0.12s ease,
    box-shadow 0.16s ease;
}

.left-nav-item:hover {
  color: var(--ink-1);
  border-color: color-mix(in srgb, var(--brand-2) 42%, var(--line) 58%);
  background: color-mix(in srgb, var(--brand-2) 10%, var(--card) 90%);
  transform: translateX(1px);
}

.left-nav-item:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--brand-2) 50%, white 50%);
  outline-offset: 1px;
}

.left-nav-item.active {
  background: linear-gradient(130deg, color-mix(in srgb, var(--brand-2) 24%, var(--card) 76%), color-mix(in srgb, var(--brand-1) 18%, var(--card) 82%));
  color: color-mix(in srgb, var(--ink-1) 92%, white 8%);
  border-color: color-mix(in srgb, var(--brand-2) 62%, var(--line) 38%);
  box-shadow: 0 8px 18px rgba(31, 97, 183, 0.2);
}

.event-panel {
  border: 1px solid var(--line);
  border-radius: 14px;
  background: var(--card);
  box-shadow:
    0 10px 26px rgba(21, 44, 88, 0.08),
    0 2px 8px rgba(21, 44, 88, 0.08);
  animation: rise-in 360ms ease-out;
  padding: 0.78rem;
  display: grid;
  gap: 0.6rem;
  height: 100%;
  min-height: 0;
  overflow-y: auto;
}

.roster-panel {
  order: 1;
}

.teams-panel {
  order: 2;
}

.event-panel h3 {
  margin: 0;
}

.event-panel h3 + .muted {
  margin-top: -0.08rem;
}

.list-main {
  display: grid;
  gap: 0.18rem;
  min-width: 0;
  flex: 1;
}

.inline-edit-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 0.35rem;
  align-items: center;
}

.inline-edit-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 0.85fr) minmax(0, 0.85fr);
  gap: 0.4rem;
}

.team-actions {
  display: flex;
  align-items: center;
  gap: 0.38rem;
}

.team-row {
  align-items: flex-start;
}

@media (max-width: 900px) {
  .event-shell {
    padding: 1rem;
  }

  .event-workspace-card {
    min-height: 0;
  }

  .event-layout {
    grid-template-columns: 1fr;
  }

  .event-panel {
    height: auto;
    overflow: visible;
  }
}

.team-row .team-actions {
  align-self: flex-start;
}

.team-assign-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.35rem;
  align-items: center;
}

.team-player-remove {
  min-width: 1.95rem;
  min-height: 1.95rem;
  padding: 0.3rem;
}

.team-player-list {
  list-style: none;
  margin: 0.1rem 0 0;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.team-player-item {
  display: inline-flex;
  align-items: center;
  gap: 0.28rem;
  padding: 0.12rem 0.38rem;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  background: color-mix(in srgb, var(--card) 88%, #eef5ff 12%);
}

.team-player-name {
  color: var(--ink-2);
  font-size: 0.82rem;
  line-height: 1;
}

.team-player-role {
  color: var(--label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  gap: 0.15rem;
}

.team-role-icon {
  font-size: 0.82rem;
}

.role-inline {
  display: inline-flex;
  align-items: center;
  gap: 0.14rem;
}

.role-inline-icon {
  font-size: 1rem;
}

.player-name {
  line-height: 1.2;
}

.player-meta-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.34rem;
}

.team-player-rank-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.team-player-empty {
  font-size: 0.9rem;
}

.entry-list,
.players-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.entry-list li,
.player-row {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-1) 8%);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
  border-radius: 10px;
  padding: 0.64rem 0.7rem;
}

.entry-list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.7rem;
}

.entry-title {
  font-weight: 800;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.player-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.65rem;
}

.player-actions {
  display: flex;
  gap: 0.45rem;
  align-items: center;
}

.roster-list {
  max-height: 320px;
  overflow: auto;
  padding-right: 0.15rem;
}

.roster-list .player-row {
  align-items: center;
}

.roster-list .player-main {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 0.28rem;
}

.roster-list .player-main strong,
.roster-list .player-main .muted {
  overflow-wrap: anywhere;
}

.roster-list .player-actions {
  min-width: 0;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.roster-list .player-actions select {
  max-width: 100%;
}

.match-list-compact {
  max-height: 380px;
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

.matchup-row {
  display: flex;
  align-items: center;
  gap: 0.42rem;
  flex-wrap: wrap;
}

.match-side-actions {
  display: grid;
  gap: 0.32rem;
}

.compact-form {
  display: grid;
  gap: 0.62rem;
}

@media (max-width: 900px) {
  .event-edit-form {
    grid-template-columns: 1fr;
  }

  .event-layout {
    grid-template-columns: 1fr;
  }

  .event-left-nav {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .compact-form {
    grid-template-columns: 1fr;
  }

  .match-item {
    flex-direction: column;
    align-items: stretch;
  }

  .match-side-actions {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .roster-list .player-row {
    flex-direction: column;
  }

  .roster-list .player-actions {
    width: 100%;
    justify-content: flex-start;
  }

  .inline-edit-grid {
    grid-template-columns: 1fr;
  }
}

@media (min-width: 901px) {
  .compact-form {
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
  }

  .compact-form > * {
    min-width: 0;
  }

  .compact-form label {
    margin: 0;
    min-width: 0;
  }

  .event-panel .player-form.compact-form {
    grid-template-columns: minmax(0, 1.2fr) minmax(0, 0.9fr) minmax(0, 0.9fr) minmax(0, 0.8fr);
  }

  .event-panel .player-form.compact-form input,
  .event-panel .player-form.compact-form select,
  .event-panel .player-form.compact-form button {
    width: 100%;
    min-width: 0;
  }
}
</style>
