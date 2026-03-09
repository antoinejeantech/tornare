<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, provide, proxyRefs, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getRankIcon, overwatchRanks } from '../lib/ranks'
import { formatOptionsForType } from '../lib/event-format'
import { useAlert } from '../lib/alerts'
import { useConfirm } from '../lib/confirm'
import { useEventStore } from '../stores/event'
import { useMatchStore } from '../stores/match'
import RosterSection from '../components/event/RosterSection.vue'
import TeamsSection from '../components/event/TeamsSection.vue'
import MatchesSection from '../components/event/MatchesSection.vue'
import OverviewSection from '../components/event/OverviewSection.vue'
import SignupRequestsSection from '../components/event/SignupRequestsSection.vue'
import overwatchLogo from '../assets/branding/overwatch-logo-gold.png'

const route = useRoute()
const router = useRouter()
const alert = useAlert()
const confirm = useConfirm()
const eventStore = useEventStore()
const matchStore = useMatchStore()

const event = ref(null)
const loadingEvent = ref(false)
const updatingEvent = ref(false)
const creatingMatch = ref(false)
const clearingBracket = ref(false)
const deletingEvent = ref(false)
const deletingMatchId = ref(null)
const addingPlayer = ref(false)
const deletingPlayers = ref({})
const creatingTeam = ref(false)
const creatingSoloTeams = ref(false)
const balancingTeams = ref(false)
const deletingTeams = ref({})
const savingPlayerTeams = ref({})
const savingPlayerEdits = ref({})
const savingTeamEdits = ref({})
const savingMatchups = ref({})
const reportingWinners = ref({})
const cancellingWinners = ref({})
const loadingSignupRequests = ref(false)
const signupRequests = ref([])
const reviewingSignupRequests = ref({})
const signupToken = ref('')
const rotatingSignupLink = ref(false)
const updatingSignupVisibility = ref(false)
const lastBalanceSummary = ref('')

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
const matchupSelections = ref({})
const editEventName = ref('')
const editEventDescription = ref('')
const editEventStartDate = ref('')
const editEventFormat = ref('5v5')
const editEventMaxPlayers = ref(10)
const validSections = ['overview', 'roster', 'teams', 'matches', 'requests', 'settings']
const activeSection = ref('overview')
const nowTick = ref(Date.now())
let startsInTimer = null

const eventId = computed(() => String(route.params.id || ''))
const canManageEvent = computed(() => Boolean(event.value?.is_owner))
const isTourneyEvent = computed(() => String(event.value?.event_type || '').toUpperCase() === 'TOURNEY')
const eventStartsInLabel = computed(() => {
  const raw = String(event.value?.start_date || '').trim()
  if (!raw) {
    return ''
  }

  const startAt = new Date(raw).getTime()
  if (Number.isNaN(startAt)) {
    return ''
  }

  const diffMs = startAt - nowTick.value
  if (Math.abs(diffMs) < 60 * 1000) {
    return 'Live now'
  }

  const absMs = Math.abs(diffMs)
  const totalMinutes = Math.round(absMs / (60 * 1000))
  const days = Math.floor(totalMinutes / (60 * 24))
  const hours = Math.floor((totalMinutes % (60 * 24)) / 60)
  const minutes = totalMinutes % 60
  const parts = []

  if (days > 0) {
    parts.push(`${days}d`)
  }
  if (hours > 0) {
    parts.push(`${hours}h`)
  }
  if (minutes > 0 || parts.length === 0) {
    parts.push(`${minutes}m`)
  }

  const readable = parts.slice(0, 2).join(' ')
  return diffMs > 0 ? `Starts in ${readable}` : `Started ${readable} ago`
})
const headerJoinRoute = computed(() => {
  if (!event.value?.public_signup_enabled) {
    return null
  }

  const token = String(event.value?.public_signup_token || signupToken.value || '').trim()
  if (!token) {
    return null
  }

  return { name: 'join-event', params: { token } }
})
const signupShareUrl = computed(() => {
  if (!signupToken.value) {
    return ''
  }

  if (typeof window === 'undefined') {
    return `/join/${signupToken.value}`
  }

  return `${window.location.origin}/join/${signupToken.value}`
})

const pendingSignupRequestCount = computed(() => {
  if (!Array.isArray(signupRequests.value)) {
    return 0
  }

  return signupRequests.value.filter((request) => {
    return String(request?.status || '').toLowerCase() === 'pending'
  }).length
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
  const maxOk = Number.isInteger(editEventMaxPlayers.value) && editEventMaxPlayers.value >= 2 && editEventMaxPlayers.value <= 99
  const allowedFormats = formatOptionsForType(event.value?.event_type)
  const formatOk = allowedFormats.includes(editEventFormat.value)
  return nameOk && maxOk && formatOk
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

function ensureOwnerAction() {
  if (canManageEvent.value) {
    return true
  }

  setError('You do not have permission for this action.')
  return false
}

function hydrateSelections() {
  if (!event.value) {
    matchupSelections.value = {}
    return
  }

  const nextMatchups = {}
  for (const match of event.value.matches) {
    nextMatchups[match.id] = {
      teamAId: match.team_a_id ? String(match.team_a_id) : '',
      teamBId: match.team_b_id ? String(match.team_b_id) : ''
    }
  }

  matchupSelections.value = nextMatchups
}

function syncEventEditDraftFromEvent() {
  if (!event.value) {
    return
  }

  editEventName.value = event.value.name || ''
  editEventDescription.value = event.value.description || ''
  editEventStartDate.value = event.value.start_date || ''
  editEventFormat.value = event.value.format || '5v5'
  editEventMaxPlayers.value = Number(event.value.max_players)
}

async function loadEvent() {
  if (!eventId.value) {
    event.value = null
    return
  }

  loadingEvent.value = true
  try {
    lastBalanceSummary.value = ''
    event.value = await eventStore.fetchEvent(eventId.value)
    syncEventEditDraftFromEvent()
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
    const [linkResult, requestsResult] = await Promise.allSettled([
      eventStore.fetchSignupLink(eventId.value),
      eventStore.listSignupRequests(eventId.value),
    ])

    if (linkResult.status === 'fulfilled') {
      signupToken.value = linkResult.value?.signup_token || ''
    } else {
      signupToken.value = ''
    }

    if (requestsResult.status === 'fulfilled') {
      signupRequests.value = Array.isArray(requestsResult.value) ? requestsResult.value : []
    } else {
      signupRequests.value = []
      reviewingSignupRequests.value = {}
    }

    if (linkResult.status === 'rejected' && requestsResult.status === 'rejected') {
      throw new Error('Failed to load signup link and requests')
    }

    if (linkResult.status === 'rejected') {
      setError('Failed to refresh signup link. Please retry before sharing.')
    } else if (requestsResult.status === 'rejected') {
      setError('Failed to load signup requests')
    }
  } catch (err) {
    signupRequests.value = []
    reviewingSignupRequests.value = {}
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

async function rotateSignupLink() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || rotatingSignupLink.value) {
    return
  }

  const confirmed = await confirm.ask({
    title: 'Rotate signup link?',
    message: 'The current shared link will stop working immediately.',
    confirmText: 'Rotate link',
    tone: 'warning',
  })
  if (!confirmed) {
    return
  }

  rotatingSignupLink.value = true
  try {
    const response = await eventStore.rotateSignupLink(eventId.value)
    signupToken.value = response.signup_token || ''
    setNotice('Signup link rotated')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to rotate signup link')
  } finally {
    rotatingSignupLink.value = false
  }
}

async function setSignupVisibility(enabled) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || updatingSignupVisibility.value) {
    return
  }

  const currentlyPublic = Boolean(event.value?.public_signup_enabled)
  if (!enabled && currentlyPublic) {
    const confirmed = await confirm.ask({
      title: 'Make registration private?',
      message: 'This hides the public Join button and rotates the signup link token. Existing shared links will stop working.',
      confirmText: 'Make private',
      tone: 'warning',
    })
    if (!confirmed) {
      return
    }
  }

  updatingSignupVisibility.value = true
  try {
    const updatedEvent = await eventStore.setSignupVisibility(eventId.value, enabled)
    event.value = updatedEvent
    hydrateSelections()
    await loadOwnerSignupData()
    setNotice(enabled ? 'Public event registration enabled' : 'Event registration is now private. Public Join button is hidden and the signup link was rotated.')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to update signup visibility')
  } finally {
    updatingSignupVisibility.value = false
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

async function autoCreateSoloTeams() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || creatingSoloTeams.value) {
    return
  }

  creatingSoloTeams.value = true
  try {
    const updatedEvent = await eventStore.autoCreateSoloTeams(eventId.value)
    event.value = updatedEvent
    hydrateSelections()
    setNotice('Created solo teams for unassigned players')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to auto-create solo teams')
  } finally {
    creatingSoloTeams.value = false
  }
}

async function autoBalanceTeams() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || balancingTeams.value) {
    return
  }

  balancingTeams.value = true
  try {
    const response = await eventStore.autoBalanceTeams(eventId.value)
    const updatedEvent = response?.event || response
    event.value = updatedEvent
    hydrateSelections()
    lastBalanceSummary.value = response?.summary || 'Teams auto-balanced by rank ELO'
    setNotice(lastBalanceSummary.value)
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to auto-balance teams')
  } finally {
    balancingTeams.value = false
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

  const confirmed = await confirm.ask({
    title: 'Delete team?',
    message: `Delete team "${team.name}"?`,
    confirmText: 'Delete team',
    tone: 'danger',
  })
  if (!confirmed) {
    return
  }

  deletingTeams.value = {
    ...deletingTeams.value,
    [team.id]: true
  }

  try {
    await eventStore.deleteTeam(eventId.value, team.id)

    if (event.value) {
      const deletedTeamId = String(team.id)

      event.value = {
        ...event.value,
        teams: event.value.teams.filter((entry) => String(entry.id) !== deletedTeamId),
        players: event.value.players.map((player) => {
          if (String(player.team_id || '') !== deletedTeamId) {
            return player
          }

          return {
            ...player,
            team_id: null,
            team: null,
          }
        }),
        matches: event.value.matches.map((match) => {
          const clearsTeamA = String(match.team_a_id || '') === deletedTeamId
          const clearsTeamB = String(match.team_b_id || '') === deletedTeamId
          const clearsWinner = String(match.winner_team_id || '') === deletedTeamId

          if (!clearsTeamA && !clearsTeamB && !clearsWinner) {
            return match
          }

          return {
            ...match,
            team_a_id: clearsTeamA ? null : match.team_a_id,
            team_a_name: clearsTeamA ? null : match.team_a_name,
            team_b_id: clearsTeamB ? null : match.team_b_id,
            team_b_name: clearsTeamB ? null : match.team_b_name,
            winner_team_id: clearsWinner ? null : match.winner_team_id,
            winner_team_name: clearsWinner ? null : match.winner_team_name,
          }
        }),
      }
    }

    if (editingTeamId.value === team.id) {
      editingTeamId.value = null
      editTeamName.value = ''
    }

    hydrateSelections()
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

async function assignPlayerToTeam(playerId, teamId) {
  await setPlayerTeam(playerId, teamId)
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

  const confirmed = await confirm.ask({
    title: 'Remove player?',
    message: `Remove player "${player.name}" from this event?`,
    confirmText: 'Remove player',
    tone: 'danger',
  })
  if (!confirmed) {
    return
  }

  deletingPlayers.value = {
    ...deletingPlayers.value,
    [player.id]: true
  }

  const previousEvent = event.value
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
    await eventStore.deletePlayer(eventId.value, player.id)

    setNotice('Player removed from event roster')
  } catch (err) {
    event.value = previousEvent
    setError(err instanceof Error ? err.message : 'Failed to remove player')
  } finally {
    deletingPlayers.value = {
      ...deletingPlayers.value,
      [player.id]: false
    }
  }
}

async function saveMatchup(matchId) {
  if (!ensureOwnerAction()) {
    return false
  }

  if (!eventId.value || savingMatchups.value[matchId]) {
    return false
  }

  const selection = matchupSelections.value[matchId] || { teamAId: '', teamBId: '' }
  const teamAId = selection.teamAId || null
  const teamBId = selection.teamBId || null

  savingMatchups.value = {
    ...savingMatchups.value,
    [matchId]: true
  }

  try {
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
    return true
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to save matchup')
    return false
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

async function generateTourneyBracket(mode = 'random') {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || !isTourneyEvent.value || creatingMatch.value) {
    return
  }

  const hasPlayedMatches = Boolean(event.value?.matches?.some((match) => Boolean(match.winner_team_id)))
  if (hasPlayedMatches) {
    setError('Cannot regenerate bracket after matches have been played')
    return
  }

  creatingMatch.value = true
  try {
    const updatedEvent = await matchStore.generateTourneyBracket(eventId.value, mode)
    event.value = updatedEvent
    hydrateSelections()
    setNotice(mode === 'empty' ? 'Empty tournament bracket generated' : 'Random tournament bracket generated')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to generate bracket')
  } finally {
    creatingMatch.value = false
  }
}

async function clearTourneyBracket() {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || !isTourneyEvent.value || clearingBracket.value) {
    return
  }

  const hasPlayedMatches = Boolean(event.value?.matches?.some((match) => Boolean(match.winner_team_id)))
  if (hasPlayedMatches) {
    setError('Cannot clear bracket after matches have been played')
    return
  }

  const hasBracketMatches = Boolean(event.value?.matches?.length)
  if (!hasBracketMatches) {
    setNotice('No generated bracket to clear')
    return
  }

  const confirmed = await confirm.ask({
    title: 'Clear bracket?',
    message: 'Delete generated bracket matches? This cannot be undone.',
    confirmText: 'Delete bracket',
    tone: 'danger',
  })
  if (!confirmed) {
    return
  }

  clearingBracket.value = true
  try {
    const updatedEvent = await matchStore.clearTourneyBracket(eventId.value)
    event.value = updatedEvent
    hydrateSelections()
    setNotice('Bracket cleared')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to clear bracket')
  } finally {
    clearingBracket.value = false
  }
}

async function reportMatchWinner(matchId, winnerTeamId) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || !isTourneyEvent.value || !winnerTeamId || reportingWinners.value[matchId]) {
    return
  }

  reportingWinners.value = {
    ...reportingWinners.value,
    [matchId]: true,
  }

  const savedWindowY = typeof window !== 'undefined' ? window.scrollY : 0
  const savedWindowX = typeof window !== 'undefined' ? window.scrollX : 0
  const savedBracketScrollLeft = typeof document !== 'undefined'
    ? document.querySelector('.tourney-bracket-wrap')?.scrollLeft ?? 0
    : 0

  try {
    await matchStore.reportMatchWinner(eventId.value, matchId, winnerTeamId)
    const updatedEvent = await eventStore.fetchEvent(eventId.value)
    event.value = updatedEvent
    hydrateSelections()
    await nextTick()

    if (typeof window !== 'undefined') {
      window.scrollTo({ top: savedWindowY, left: savedWindowX })
    }

    if (typeof document !== 'undefined') {
      const bracketWrap = document.querySelector('.tourney-bracket-wrap')
      if (bracketWrap) {
        bracketWrap.scrollLeft = savedBracketScrollLeft
      }
    }

    setNotice('Winner reported')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to report winner')
  } finally {
    reportingWinners.value = {
      ...reportingWinners.value,
      [matchId]: false,
    }
  }
}

async function cancelMatchWinner(matchId) {
  if (!ensureOwnerAction()) {
    return
  }

  if (!eventId.value || !isTourneyEvent.value || cancellingWinners.value[matchId]) {
    return
  }

  const confirmed = await confirm.ask({
    title: 'Cancel match result?',
    message: 'Downstream bracket progression will be reset where needed.',
    confirmText: 'Cancel result',
    tone: 'warning',
  })
  if (!confirmed) {
    return
  }

  cancellingWinners.value = {
    ...cancellingWinners.value,
    [matchId]: true,
  }

  const savedWindowY = typeof window !== 'undefined' ? window.scrollY : 0
  const savedWindowX = typeof window !== 'undefined' ? window.scrollX : 0
  const savedBracketScrollLeft = typeof document !== 'undefined'
    ? document.querySelector('.tourney-bracket-wrap')?.scrollLeft ?? 0
    : 0

  try {
    await matchStore.cancelMatchWinner(eventId.value, matchId)
    const updatedEvent = await eventStore.fetchEvent(eventId.value)
    event.value = updatedEvent
    hydrateSelections()
    await nextTick()

    if (typeof window !== 'undefined') {
      window.scrollTo({ top: savedWindowY, left: savedWindowX })
    }

    if (typeof document !== 'undefined') {
      const bracketWrap = document.querySelector('.tourney-bracket-wrap')
      if (bracketWrap) {
        bracketWrap.scrollLeft = savedBracketScrollLeft
      }
    }

    setNotice('Match result cancelled')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to cancel match result')
  } finally {
    cancellingWinners.value = {
      ...cancellingWinners.value,
      [matchId]: false,
    }
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
  const confirmed = await confirm.ask({
    title: 'Delete match?',
    message: `Delete match "${target?.title || matchId}"?`,
    confirmText: 'Delete match',
    tone: 'danger',
  })
  if (!confirmed) {
    return
  }

  deletingMatchId.value = matchId
  try {
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

  const confirmed = await confirm.ask({
    title: 'Delete event?',
    message: `Delete event "${event.value.name}" and all its matches?`,
    confirmText: 'Delete event',
    tone: 'danger',
  })
  if (!confirmed) {
    return
  }

  deletingEvent.value = true
  try {
    await eventStore.deleteEvent(eventId.value)

    router.push({ name: 'home' })
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to delete event')
  } finally {
    deletingEvent.value = false
  }
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
    const payloadType = String(event.value.event_type).trim().toUpperCase() === 'TOURNEY' ? 'TOURNEY' : 'PUG'

    const updatedEvent = await eventStore.updateEvent(eventId.value, {
      name: editEventName.value.trim(),
      description: editEventDescription.value.trim(),
      start_date: editEventStartDate.value ? editEventStartDate.value : null,
      event_type: payloadType,
      format: editEventFormat.value,
      max_players: editEventMaxPlayers.value,
    })

    event.value = updatedEvent
    syncEventEditDraftFromEvent()
    hydrateSelections()
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

function normalizeSection(section) {
  const candidate = String(section || '').trim().toLowerCase()
  if (!validSections.includes(candidate)) {
    return 'overview'
  }

  if ((candidate === 'requests' || candidate === 'settings') && !canManageEvent.value) {
    return 'overview'
  }

  return candidate
}

function openSection(section) {
  const nextSection = normalizeSection(section)
  const currentSection = normalizeSection(route.query.section)
  if (nextSection === currentSection) {
    activeSection.value = nextSection
    return
  }

  router.push({
    name: 'event',
    params: { id: eventId.value },
    query: {
      ...route.query,
      section: nextSection,
    },
  })
}

watch(
  () => route.params.id,
  () => {
    loadEvent()
  }
)

watch(
  () => route.query.section,
  (section) => {
    activeSection.value = normalizeSection(section)
  },
  { immediate: true }
)

watch(
  canManageEvent,
  () => {
    const normalizedSection = normalizeSection(route.query.section)
    activeSection.value = normalizedSection

    if (String(route.query.section || '') !== normalizedSection) {
      router.replace({
        name: 'event',
        params: { id: eventId.value },
        query: {
          ...route.query,
          section: normalizedSection,
        },
      })
    }
  }
)

onMounted(() => {
  loadEvent()
  startsInTimer = window.setInterval(() => {
    nowTick.value = Date.now()
  }, 30 * 1000)
})

onBeforeUnmount(() => {
  if (startsInTimer) {
    window.clearInterval(startsInTimer)
  }
})

provide('eventCtx', proxyRefs({
  event,
  eventIsFull,
  loadingEvent,
  creatingTeam,
  creatingSoloTeams,
  balancingTeams,
  creatingMatch,
  clearingBracket,
  deletingEvent,
  deletingMatchId,
  addingPlayer,
  deletingPlayers,
  deletingTeams,
  savingPlayerTeams,
  savingPlayerEdits,
  savingTeamEdits,
  savingMatchups,
  reportingWinners,
  cancellingWinners,
  isTourneyEvent,
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
  matchupSelections,
  canCreateTeam,
  canCreateMatch,
  canAddPlayer,
  canManageEvent,
  signupRequests,
  loadingSignupRequests,
  reviewingSignupRequests,
  rotatingSignupLink,
  updatingSignupVisibility,
  signupShareUrl,
  signupToken,
  lastBalanceSummary,
  openSection,
  createTeam,
  autoCreateSoloTeams,
  autoBalanceTeams,
  createMatch,
  generateTourneyBracket,
  clearTourneyBracket,
  deleteEvent,
  deleteMatch,
  saveMatchup,
  reportMatchWinner,
  cancelMatchWinner,
  saveTeamEdit,
  deleteTeam,
  assignPlayerToTeam,
  removePlayerFromTeam,
  savePlayerEdit,
  addPlayer,
  removePlayer,
  copySignupLink,
  rotateSignupLink,
  setSignupVisibility,
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
        <div class="event-title-stack">
          <img class="event-logo" :src="overwatchLogo" alt="Overwatch" />
          <div class="event-title-row">
            <h2>{{ event.name }}</h2>
            <p v-if="eventStartsInLabel" class="event-starts-in muted">{{ eventStartsInLabel }}</p>
          </div>
        </div>
        <div class="event-header-actions">
          <RouterLink v-if="headerJoinRoute" class="btn-primary event-join-header-btn" :to="headerJoinRoute">
            Join event
          </RouterLink>
        </div>
      </div>

      <div class="event-layout">
        <aside class="event-left-nav" aria-label="Event sections">
          <button class="left-nav-item" :class="{ active: activeSection === 'overview' }" @click="openSection('overview')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">dashboard</span>
              <span>Overview</span>
            </span>
          </button>
          <button class="left-nav-item" :class="{ active: activeSection === 'roster' }" @click="openSection('roster')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">groups</span>
              <span>Players</span>
            </span>
          </button>
          <button class="left-nav-item" :class="{ active: activeSection === 'teams' }" @click="openSection('teams')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">shield</span>
              <span>Teams</span>
            </span>
          </button>
          <button class="left-nav-item" :class="{ active: activeSection === 'matches' }" @click="openSection('matches')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">sports_score</span>
              <span>Matches</span>
            </span>
          </button>
          <button v-if="canManageEvent" class="left-nav-item" :class="{ active: activeSection === 'requests' }" @click="openSection('requests')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">mail</span>
              <span>Requests</span>
            </span>
            <span v-if="pendingSignupRequestCount > 0" class="left-nav-badge" :aria-label="`${pendingSignupRequestCount} pending signup requests`">
              {{ pendingSignupRequestCount }}
            </span>
          </button>
          <button v-if="canManageEvent" class="left-nav-item" :class="{ active: activeSection === 'settings' }" @click="openSection('settings')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">settings</span>
              <span>Settings</span>
            </span>
          </button>
        </aside>

        <section class="event-panel">
          <OverviewSection v-if="activeSection === 'overview'" />
          <RosterSection v-else-if="activeSection === 'roster'" />
          <TeamsSection v-else-if="activeSection === 'teams'" />
          <MatchesSection v-else-if="activeSection === 'matches'" />
          <SignupRequestsSection v-else-if="activeSection === 'requests' && canManageEvent" />
          <section v-else-if="activeSection === 'settings' && canManageEvent" class="event-settings-section">
            <h3 class="section-title">
              <span class="material-symbols-rounded section-title-icon" aria-hidden="true">settings</span>
              <span>Settings</span>
            </h3>

            <div class="event-registration-toggle-box" :class="event.public_signup_enabled ? 'is-public' : 'is-private'">
              <div class="event-registration-header">
                <p class="event-registration-kicker">Event registration</p>
                <span class="event-registration-state-pill" :class="event.public_signup_enabled ? 'is-public' : 'is-private'">
                  {{ event.public_signup_enabled ? 'Public' : 'Private' }}
                </span>
              </div>

              <p class="event-registration-copy">
                {{ event.public_signup_enabled
                  ? 'Anyone can discover this event and use the Join button from event surfaces.'
                  : 'Only people with a direct invite link can submit a signup request.' }}
              </p>

              <div class="event-registration-toggle-actions">
                <button
                  class="btn-secondary"
                  :disabled="updatingSignupVisibility"
                  @click="setSignupVisibility(!event.public_signup_enabled)"
                >
                  {{ updatingSignupVisibility ? 'Updating...' : (event.public_signup_enabled ? 'Make private' : 'Make public') }}
                </button>
              </div>

              <p class="muted event-registration-note">
                {{ event.public_signup_enabled
                  ? 'Switching to private hides the public Join button and rotates the signup token. Existing shared links stop working.'
                  : 'Switch to public to show the Join button to everyone.' }}
              </p>
            </div>

            <form class="event-edit-form" @submit.prevent="saveEventEdit">
              <label>
                Event name
                <input v-model="editEventName" placeholder="Event name" />
              </label>
              <label>
                Description
                <textarea v-model="editEventDescription" rows="4" placeholder="Rules, cashprize, check-in info..." />
              </label>
              <label>
                Start date
                <input v-model="editEventStartDate" type="datetime-local" />
              </label>
              <label>
                Format
                <select v-model="editEventFormat">
                  <option
                    v-for="format in formatOptionsForType(event.event_type)"
                    :key="`edit-event-format-${format}`"
                    :value="format"
                  >
                    {{ format }}
                  </option>
                </select>
              </label>
              <label>
                Max players
                <input v-model.number="editEventMaxPlayers" type="number" min="2" max="99" step="1" />
              </label>

              <div class="event-settings-actions">
                <button class="btn-primary" :disabled="updatingEvent || !canSaveEventMeta" type="submit">
                  {{ updatingEvent ? 'Saving...' : 'Save event settings' }}
                </button>
                <button class="btn-secondary" :disabled="updatingEvent" type="button" @click="syncEventEditDraftFromEvent">
                  Reset changes
                </button>
                <button class="btn-danger" :disabled="deletingEvent || updatingEvent" type="button" @click="deleteEvent">
                  {{ deletingEvent ? 'Deleting event...' : 'Delete event' }}
                </button>
              </div>
            </form>
          </section>
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
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.7rem;
}

.event-shell {
  max-width: none;
  width: 100%;
  padding: 1.1rem 1.2rem 1.25rem;
}

.event-workspace-card {
  display: flex;
  flex-direction: column;
}

.event-header-row h2 {
  margin: 0;
  text-transform: capitalize;
}

.event-title-stack {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  grid-template-rows: auto;
  gap: 0.42rem 0.6rem;
  min-width: 0;
}

.event-title-row {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.55rem;
  min-width: 0;
  grid-column: 2;
  grid-row: 1;
}

.event-starts-in {
  margin: 0;
  font-size: 0.82rem;
  font-weight: 600;
}

.event-logo {
  width: 4.5rem;
  height: 4.5rem;
  border-radius: 8px;
  object-fit: contain;
  background: color-mix(in srgb, var(--card) 74%, #19253a 26%);
  box-shadow: 0 3px 10px rgba(17, 52, 112, 0.16);
  padding: 0.2rem;
  grid-column: 1;
  grid-row: 1;
  align-self: stretch;
}

.event-header-actions {
  display: flex;
  gap: 0.35rem;
  align-items: center;
  align-self: center;
  flex-wrap: wrap;
}

.event-join-header-btn {
  text-decoration: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 1.05rem;
  font-weight: 400;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-radius: 8px;
  padding: 0.56rem 1.02rem;
  box-shadow: 0 10px 22px rgba(123, 89, 30, 0.36);
}

.event-edit-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 0.5rem;
  margin: 0.55rem 0 0.7rem;
}

.event-edit-form label {
  display: grid;
  gap: 0.24rem;
}

.event-settings-section {
  display: grid;
  gap: 0.62rem;
}

.event-registration-toggle-box {
  border: 1px solid color-mix(in srgb, var(--line) 72%, var(--brand-2) 28%);
  border-radius: 14px;
  padding: 0.82rem;
  background:
    radial-gradient(140px 90px at 100% 0%, color-mix(in srgb, var(--brand-2) 18%, transparent 82%), transparent 72%),
    linear-gradient(160deg, color-mix(in srgb, var(--card) 90%, #eef5ff 10%), color-mix(in srgb, var(--card) 96%, #f7faff 4%));
  display: grid;
  gap: 0.62rem;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);
}

.event-registration-toggle-box.is-private {
  border-color: color-mix(in srgb, #e36b55 26%, var(--line) 74%);
  background:
    radial-gradient(120px 85px at 100% 0%, rgba(227, 107, 85, 0.13), transparent 72%),
    linear-gradient(160deg, color-mix(in srgb, var(--card) 90%, #fff0ec 10%), color-mix(in srgb, var(--card) 96%, #fff8f6 4%));
}

.event-registration-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
}

.event-registration-kicker {
  margin: 0;
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 700;
  color: color-mix(in srgb, var(--ink-2) 82%, var(--brand-1) 18%);
}

.event-registration-state-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  padding: 0.22rem 0.62rem;
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border: 1px solid transparent;
}

.event-registration-state-pill.is-public {
  color: #edfdf4;
  background: color-mix(in srgb, #1c7a4f 82%, #0f2d1f 18%);
  border-color: color-mix(in srgb, #36b376 58%, #0f2d1f 42%);
}

.event-registration-state-pill.is-private {
  color: #fff3f1;
  background: color-mix(in srgb, #8f3427 84%, #2c1411 16%);
  border-color: color-mix(in srgb, #d96a57 56%, #2c1411 44%);
}

.event-registration-copy {
  margin: 0;
  font-size: 0.9rem;
  color: var(--ink-1);
}

.event-registration-toggle-actions {
  display: flex;
  justify-content: flex-start;
}

.event-registration-note {
  margin: 0;
  font-size: 0.84rem;
  line-height: 1.35;
}

.event-settings-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.45rem;
}

@media (max-width: 720px) {
  .event-registration-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .event-registration-toggle-actions,
  .event-registration-toggle-actions button {
    width: 100%;
  }

  .event-settings-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .event-settings-actions button {
    width: 100%;
  }
}

.section-title {
  margin: 0;
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
}

.section-title-icon {
  font-size: 1.12rem;
  line-height: 1;
}

.event-layout {
  display: grid;
  grid-template-columns: 200px minmax(0, 1fr);
  gap: 0.75rem;
  align-items: start;
  margin-bottom: 0;
}

.event-left-nav {
  position: sticky;
  top: 5.1rem;
  display: grid;
  gap: 0.34rem;
  border: 1px solid color-mix(in srgb, var(--brand-1) 30%, var(--line) 70%);
  border-radius: 14px;
  padding: 0.52rem;
  background:
    radial-gradient(180px 80px at 8% 0%, color-mix(in srgb, var(--brand-2) 14%, transparent 86%), transparent 72%),
    linear-gradient(180deg, color-mix(in srgb, var(--card) 88%, #edf4ff 12%), color-mix(in srgb, var(--card) 96%, #f4f8ff 4%));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.06),
    0 6px 16px rgba(16, 39, 82, 0.12);
  align-self: start;
  height: fit-content;
}

.left-nav-item {
  position: relative;
  overflow: hidden;
  width: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  text-align: left;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  background: color-mix(in srgb, var(--card) 92%, #f4f8ff 8%);
  color: var(--ink-2);
  border-radius: 10px;
  padding: 0.55rem 0.62rem;
  font-weight: 700;
  letter-spacing: 0.01em;
  cursor: pointer;
  transition:
    background 0.16s ease,
    border-color 0.16s ease,
    color 0.16s ease,
    transform 0.12s ease,
    box-shadow 0.16s ease;
}

.left-nav-label {
  display: inline-flex;
  align-items: center;
  gap: 0.38rem;
}

.left-nav-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.45rem;
  height: 1.45rem;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--line) 74%, var(--brand-2) 26%);
  background: color-mix(in srgb, var(--card) 88%, #eff5ff 12%);
  color: color-mix(in srgb, var(--brand-1) 72%, var(--ink-1) 28%);
  font-size: 0.95rem;
  line-height: 1;
  transition:
    background 0.16s ease,
    border-color 0.16s ease,
    color 0.16s ease,
    transform 0.16s ease;
}

.left-nav-item:hover {
  color: var(--ink-1);
  border-color: color-mix(in srgb, var(--brand-2) 42%, var(--line) 58%);
  background: color-mix(in srgb, var(--brand-2) 10%, var(--card) 90%);
  transform: translateX(2px);
}

.left-nav-item:hover .left-nav-icon {
  border-color: color-mix(in srgb, var(--brand-2) 56%, var(--line) 44%);
  background: color-mix(in srgb, var(--brand-2) 20%, var(--card) 80%);
  color: color-mix(in srgb, var(--ink-1) 88%, var(--brand-1) 12%);
  transform: translateY(-1px);
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

.left-nav-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 6px;
  bottom: 6px;
  width: 3px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--brand-1) 84%, #fff 16%);
}

.left-nav-item.active .left-nav-icon {
  border-color: color-mix(in srgb, var(--brand-1) 68%, var(--line) 32%);
  background: color-mix(in srgb, var(--brand-1) 28%, var(--card) 72%);
  color: color-mix(in srgb, var(--ink-1) 96%, #fff 4%);
}

.left-nav-badge {
  min-width: 1.35rem;
  height: 1.35rem;
  padding: 0 0.35rem;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, #ff5a3d 86%, white 14%);
  color: white;
  font-size: 0.74rem;
  font-weight: 800;
  line-height: 1;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.16);
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

  .event-left-nav {
    position: static;
    top: auto;
  }

  .event-header-row {
    flex-direction: column;
    align-items: stretch;
  }

  .event-header-actions {
    align-self: flex-end;
  }

  .event-title-stack {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto;
    gap: 0.35rem;
  }

  .event-logo {
    width: 3.4rem;
    height: 3.4rem;
    grid-column: 1;
    grid-row: 1;
    align-self: start;
  }

  .event-title-row {
    grid-column: 1;
    grid-row: 2;
  }

  .event-panel {
    height: auto;
    overflow: visible;
  }
}
</style>
