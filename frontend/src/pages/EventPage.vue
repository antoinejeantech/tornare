<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, provide, proxyRefs, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { getDateTimestamp, parseDateValue } from '../lib/dates'
import { getRankIcon, overwatchRanks } from '../lib/ranks'
import { usePageRevalidation } from '../composables/usePageRevalidation'
import { useAlert } from '../composables/alerts'
import { useConfirm } from '../composables/confirm'
import { useAuthStore } from '../stores/auth'
import { useEventStore } from '../stores/event'
import { useMatchStore } from '../stores/match'
import { useEventSettings } from '../composables/event/useEventSettings'
import { useEventSignup } from '../composables/event/useEventSignup'
import { useEventTeams } from '../composables/event/useEventTeams'
import { useEventPlayers } from '../composables/event/useEventPlayers'
import { useEventMatches } from '../composables/event/useEventMatches'
import RosterSection from '../components/event/RosterSection.vue'
import TeamsSection from '../components/event/TeamsSection.vue'
import MatchesSection from '../components/event/MatchesSection.vue'
import OverviewSection from '../components/event/OverviewSection.vue'
import SignupRequestsSection from '../components/event/SignupRequestsSection.vue'
import SettingsSection from '../components/event/SettingsSection.vue'
import ActionCtaButton from '../components/ui/ActionCtaButton.vue'
import AppButton from '../components/ui/AppButton.vue'
import AppBadge from '../components/ui/AppBadge.vue'
import type { Event } from '../types'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const alert = useAlert()
const confirm = useConfirm()
const authStore = useAuthStore()
const eventStore = useEventStore()
const matchStore = useMatchStore()

// ── Core state ─────────────────────────────────────────────────────────
const event = ref<Event | null>(null)
const loadingEvent = ref(false)
const matchupSelections = ref<Record<string, { teamAId: string; teamBId: string }>>({})
const nowTick = ref(Date.now())
let startsInTimer: ReturnType<typeof setInterval> | null = null
let latestEventLoadRequestId = 0
let eventLoadController: AbortController | null = null

// ── Core computeds ─────────────────────────────────────────────────────
const eventId = computed(() => String(route.params.id || ''))
const canManageEvent = computed(() => Boolean(event.value?.can_manage))
const hasEventAdminAccess = computed(() => {
  const role = String(authStore.user?.role || '').toLowerCase()
  return role === 'admin' || role === 'moderator'
})
const authIdentityKey = computed(() => {
  const initialized = authStore.initialized ? '1' : '0'
  const authenticated = authStore.isAuthenticated ? '1' : '0'
  const userId = String(authStore.user?.id || '')
  return `${initialized}:${authenticated}:${userId}`
})
const isTourneyEvent = computed(() => String(event.value?.event_type || '').toUpperCase() === 'TOURNEY')
const eventStartsInLabel = computed(() => {
  const raw = String(event.value?.start_date || '').trim()
  if (!raw) return ''
  const startAt = getDateTimestamp(raw)
  if (startAt === null) return ''
  const diffMs = startAt - nowTick.value
  if (Math.abs(diffMs) < 60 * 1000) return t('eventPage.liveNow')
  const absMs = Math.abs(diffMs)
  const totalMinutes = Math.round(absMs / (60 * 1000))
  const days = Math.floor(totalMinutes / (60 * 24))
  const hours = Math.floor((totalMinutes % (60 * 24)) / 60)
  const minutes = totalMinutes % 60
  const parts = []
  if (days > 0) parts.push(`${days}d`)
  if (hours > 0) parts.push(`${hours}h`)
  if (minutes > 0 || parts.length === 0) parts.push(`${minutes}m`)
  const readable = parts.slice(0, 2).join(' ')
  return diffMs > 0 ? t('eventPage.startsIn', { time: readable }) : t('eventPage.startedAgo', { time: readable })
})
const eventStartDateTimeLabel = computed(() => {
  const raw = String(event.value?.start_date || '').trim()
  if (!raw) return ''
  const parsed = parseDateValue(raw)
  if (!parsed) return ''
  const day = String(parsed.getDate()).padStart(2, '0')
  const month = String(parsed.getMonth() + 1).padStart(2, '0')
  const year = String(parsed.getFullYear())
  const hours = String(parsed.getHours()).padStart(2, '0')
  const minutes = String(parsed.getMinutes()).padStart(2, '0')
  return `${day}/${month}/${year} ${hours}:${minutes}`
})
const eventIsFull = computed(() => {
  if (!event.value) return false
  return event.value.players.length >= event.value.max_players
})

// ── Utilities ──────────────────────────────────────────────────────────
function setError(message: string) { alert.error(message) }
function setNotice(message: string) { alert.success(message) }
function ensureOwnerAction() {
  if (canManageEvent.value) return true
  setError(t('eventPage.noPermission'))
  return false
}
function hydrateSelections() {
  if (!event.value) { matchupSelections.value = {}; return }
  const nextMatchups: Record<string | number, { teamAId: string; teamBId: string }> = {}
  for (const match of event.value.matches) {
    nextMatchups[match.id] = {
      teamAId: match.team_a_id ? String(match.team_a_id) : '',
      teamBId: match.team_b_id ? String(match.team_b_id) : '',
    }
  }
  matchupSelections.value = nextMatchups
}

// ── Domain composables ─────────────────────────────────────────────────
const sharedCtx = { event, eventId, canManageEvent, ensureOwnerAction, setError, setNotice, hydrateSelections, eventStore, confirm }
const settings = useEventSettings({ ...sharedCtx, router })
const signup = useEventSignup({ ...sharedCtx, hasEventAdminAccess })
const teams = useEventTeams(sharedCtx)
const players = useEventPlayers({ ...sharedCtx, eventIsFull, clearLastBalancedFingerprint: teams.clearLastBalancedFingerprint })
const matches = useEventMatches({ ...sharedCtx, isTourneyEvent, matchupSelections, matchStore })

// ── Destructure composable returns ─────────────────────────────────────
const {
  editEventName, editEventDescription, editEventStartDate, editEventFormat, editEventMaxPlayers,
  editEventRequireDiscord, editEventRequireBattletag,
  updatingEvent, deletingEvent, canSaveEventMeta,
  syncEventEditDraftFromEvent, saveEventEdit, deleteEvent,
} = settings

const {
  signupToken, signupRequests, loadingSignupRequests, reviewingSignupRequests,
  rotatingSignupLink, updatingSignupVisibility, updatingFeaturedEvent, updatingEventStatus,
  signupShareUrl, pendingSignupRequestCount,
  loadOwnerSignupData, clearSignupData, copySignupLink, rotateSignupLink, setSignupVisibility,
  setFeaturedEvent, publishEvent, unpublishEvent, endEvent, acceptSignupRequest, declineSignupRequest,
} = signup

const {
  creatingTeam, creatingSoloTeams, balancingTeams, deletingTeams, savingTeamEdits,
  newTeamName, editingTeamId, editTeamName, canCreateTeam,
  lastBalanceSummary, teamsAreAlreadyBalanced,
  createTeam, autoCreateSoloTeams, autoBalanceTeams, saveTeamEdit, deleteTeam,
} = teams

const {
  addingPlayer, deletingPlayers, savingPlayerEdits, savingPlayerTeams,
  newPlayerName, newPlayerRole, newPlayerRank, newPlayerRoles,
  editingPlayerId, editPlayerName, editPlayerRole, editPlayerRank, editPlayerRoles,
  canAddPlayer,
  addPlayer, savePlayerEdit, assignPlayerToTeam, assignPlayerToTeamWithRole,
  removePlayerFromTeam, removePlayer,
} = players

const {
  creatingMatch, clearingBracket, deletingMatchId, savingMatchups, reportingWinners, cancellingWinners,
  newMatchTitle, newMatchMap, newMatchTeamAId, newMatchTeamBId, newMatchStartDate, canCreateMatch,
  initializeNewMatchDraft,
  saveMatchup, createMatch, updateMatchStartDate, generateTourneyBracket, clearTourneyBracket,
  reportMatchWinner, cancelMatchWinner, deleteMatch,
} = matches

// ── Event loading ──────────────────────────────────────────────────────
async function loadEvent() {
  if (!eventId.value) { event.value = null; return }
  if (eventLoadController) eventLoadController.abort()
  eventLoadController = new AbortController()
  const requestId = ++latestEventLoadRequestId
  loadingEvent.value = true
  try {
    teams.lastBalanceSummary.value = ''
    const nextEvent = await eventStore.fetchEvent(eventId.value, { signal: eventLoadController.signal })
    if (requestId !== latestEventLoadRequestId) return
    event.value = nextEvent
    syncEventEditDraftFromEvent()
    hydrateSelections()
    if (canManageEvent.value) {
      await loadOwnerSignupData()
    } else {
      clearSignupData()
    }
  } catch (err) {
    if (err instanceof Error && err.name === 'AbortError') return
    if (requestId !== latestEventLoadRequestId) return
    event.value = null
    setError(err instanceof Error ? err.message : t('eventPage.loadFailed'))
  } finally {
    if (requestId === latestEventLoadRequestId) loadingEvent.value = false
  }
}

// ── Section navigation ─────────────────────────────────────────────────
const validSections = ['overview', 'roster', 'teams', 'matches', 'requests', 'settings']
const activeSection = ref('overview')

function normalizeSection(section: string): string {
  const candidate = String(section || '').trim().toLowerCase()
  if (!validSections.includes(candidate)) return 'overview'
  if ((candidate === 'requests' || candidate === 'settings') && !canManageEvent.value) return 'overview'
  return candidate
}

function openSection(section: string) {
  const nextSection = normalizeSection(section)
  const currentSection = normalizeSection(String(route.query.section || ''))
  if (nextSection === currentSection) { activeSection.value = nextSection; return }
  router.push({ name: 'event', params: { id: eventId.value }, query: { ...route.query, section: nextSection } })
}

function navigateToHome() { router.push({ name: 'home' }) }

// ── Watches + lifecycle ────────────────────────────────────────────────
watch(
  () => route.query.section,
  (section) => {
    activeSection.value = normalizeSection(String(section || ''))
  },
  { immediate: true }
)

watch(
  canManageEvent,
  () => {
    const normalizedSection = normalizeSection(String(route.query.section || ''))
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

watch(
  [eventId, authIdentityKey],
  ([nextEventId], [previousEventId, previousAuthIdentityKey]) => {
    if (!authStore.initialized) {
      return
    }

    if (!nextEventId) {
      return
    }

    if (nextEventId === previousEventId && authIdentityKey.value === previousAuthIdentityKey) {
      return
    }

    loadEvent()
  },
  { immediate: true }
)

usePageRevalidation(() => loadEvent())

onMounted(() => {
  startsInTimer = window.setInterval(() => {
    nowTick.value = Date.now()
  }, 30 * 1000)
})

onBeforeUnmount(() => {
  if (eventLoadController) {
    eventLoadController.abort()
  }
  if (startsInTimer) {
    window.clearInterval(startsInTimer)
  }
})

// ── headerJoinRoute (needs both event + signupToken) ───────────────────
const headerJoinRoute = computed(() => {
  if (!event.value?.public_signup_enabled) return null
  const token = String(event.value?.public_signup_token || signupToken.value || '').trim()
  if (!token) return null
  return { name: 'join-event', params: { token } }
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
  newMatchTeamAId,
  newMatchTeamBId,
  newMatchStartDate,
  newPlayerName,
  newPlayerRole,
  newPlayerRank,
  newPlayerRoles,
  editTeamName,
  editingTeamId,
  editPlayerName,
  editPlayerRole,
  editPlayerRank,
  editPlayerRoles,
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
  updatingEvent,
  updatingEventStatus,
  signupShareUrl,
  signupToken,
  lastBalanceSummary,
  teamsAreAlreadyBalanced,
  editEventName,
  editEventDescription,
  editEventStartDate,
  editEventFormat,
  editEventMaxPlayers,
  editEventRequireDiscord,
  editEventRequireBattletag,
  canSaveEventMeta,
  openSection,
  createTeam,
  autoCreateSoloTeams,
  autoBalanceTeams,
  initializeNewMatchDraft,
  createMatch,
  updateMatchStartDate,
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
  assignPlayerToTeamWithRole,
  removePlayerFromTeam,
  savePlayerEdit,
  addPlayer,
  removePlayer,
  copySignupLink,
  rotateSignupLink,
  setSignupVisibility,
  syncEventEditDraftFromEvent,
  saveEventEdit,
  publishEvent,
  unpublishEvent,
  endEvent,
  acceptSignupRequest,
  declineSignupRequest,
  getRankIcon,
  overwatchRanks,
}))
</script>

<template>
  <main class="app-shell app-shell--wide event-shell">
    <section v-if="loadingEvent" class="event-loading-state">
      <p>{{ t('eventPage.loading') }}</p>
    </section>

    <section v-else-if="event" class="event-workspace-card">
      <div class="event-layout">
        <aside class="event-left-nav" :aria-label="t('eventPage.nav')">
          <p class="event-left-nav-kicker">{{ t('eventPage.nav') }}</p>
          <button class="left-nav-item" :class="{ active: activeSection === 'overview' }" @click="openSection('overview')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">dashboard</span>
              <span>{{ t('eventPage.sectionOverview') }}</span>
            </span>
          </button>
          <button class="left-nav-item" :class="{ active: activeSection === 'roster' }" @click="openSection('roster')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">group</span>
              <span>{{ t('eventPage.sectionPlayers') }}</span>
            </span>
          </button>
          <button class="left-nav-item" :class="{ active: activeSection === 'teams' }" @click="openSection('teams')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">verified_user</span>
              <span>{{ t('eventPage.sectionTeams') }}</span>
            </span>
          </button>
          <button class="left-nav-item" :class="{ active: activeSection === 'matches' }" @click="openSection('matches')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">swords</span>
              <span>{{ t('eventPage.sectionMatches') }}</span>
            </span>
          </button>
          <button v-if="canManageEvent" class="left-nav-item" :class="{ active: activeSection === 'requests' }" @click="openSection('requests')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">mail</span>
              <span>{{ t('eventPage.sectionRequests') }}</span>
            </span>
            <span v-if="pendingSignupRequestCount > 0" class="left-nav-badge" :aria-label="t('eventPage.pendingBadge', { count: pendingSignupRequestCount })">
              {{ pendingSignupRequestCount }}
            </span>
          </button>
          <button v-if="canManageEvent" class="left-nav-item" :class="{ active: activeSection === 'settings' }" @click="openSection('settings')">
            <span class="left-nav-label">
              <span class="material-symbols-rounded left-nav-icon" aria-hidden="true">settings</span>
              <span>{{ t('eventPage.sectionSettings') }}</span>
            </span>
          </button>
        </aside>

        <!-- Mobile bottom tab bar -->
        <nav class="event-bottom-nav" :aria-label="t('eventPage.nav')">
          <button class="bottom-nav-item" :class="{ active: activeSection === 'overview' }" @click="openSection('overview')">
            <span class="material-symbols-rounded bottom-nav-icon" aria-hidden="true">dashboard</span>
            <span class="bottom-nav-label">{{ t('eventPage.sectionOverview') }}</span>
          </button>
          <button class="bottom-nav-item" :class="{ active: activeSection === 'roster' }" @click="openSection('roster')">
            <span class="material-symbols-rounded bottom-nav-icon" aria-hidden="true">group</span>
            <span class="bottom-nav-label">{{ t('eventPage.sectionPlayers') }}</span>
          </button>
          <button class="bottom-nav-item" :class="{ active: activeSection === 'teams' }" @click="openSection('teams')">
            <span class="material-symbols-rounded bottom-nav-icon" aria-hidden="true">verified_user</span>
            <span class="bottom-nav-label">{{ t('eventPage.sectionTeams') }}</span>
          </button>
          <button class="bottom-nav-item" :class="{ active: activeSection === 'matches' }" @click="openSection('matches')">
            <span class="material-symbols-rounded bottom-nav-icon" aria-hidden="true">swords</span>
            <span class="bottom-nav-label">{{ t('eventPage.sectionMatches') }}</span>
          </button>
          <button v-if="canManageEvent" class="bottom-nav-item" :class="{ active: activeSection === 'requests' }" @click="openSection('requests')">
            <span class="material-symbols-rounded bottom-nav-icon" aria-hidden="true">mail</span>
            <span class="bottom-nav-label">{{ t('eventPage.sectionRequests') }}</span>
            <span v-if="pendingSignupRequestCount > 0" class="bottom-nav-badge" aria-hidden="true">{{ pendingSignupRequestCount }}</span>
          </button>
          <button v-if="canManageEvent" class="bottom-nav-item" :class="{ active: activeSection === 'settings' }" @click="openSection('settings')">
            <span class="material-symbols-rounded bottom-nav-icon" aria-hidden="true">settings</span>
            <span class="bottom-nav-label">{{ t('eventPage.sectionSettings') }}</span>
          </button>
        </nav>

        <section class="event-main-column">
          <div class="card event-header-card">
            <div class="event-header-top">
              <span class="event-logo" aria-hidden="true">
                <span class="material-symbols-rounded event-logo-icon" aria-hidden="true">trophy</span>
              </span>
              <div class="event-title-row">
                <div class="event-title-name-row">
                  <h2>{{ event.name }}</h2>
                  <AppBadge v-if="event.status === 'ENDED'" variant="muted" :label="t('eventPage.statusEnded')" />
                  <AppBadge v-else-if="event.status === 'DRAFT'" variant="warning" :label="t('eventPage.statusDraft')" />
                </div>
                <div v-if="eventStartsInLabel || eventStartDateTimeLabel" class="event-starts-in muted">
                  <span v-if="eventStartsInLabel" class="event-start-meta event-starts-in-countdown">
                    <span class="material-symbols-rounded" aria-hidden="true">timer</span>
                    <span>{{ eventStartsInLabel }}</span>
                  </span>
                  <span v-if="eventStartsInLabel && eventStartDateTimeLabel" class="event-start-separator event-starts-in-countdown" aria-hidden="true">|</span>
                  <span v-if="eventStartDateTimeLabel" class="event-start-meta">
                    <span class="material-symbols-rounded" aria-hidden="true">calendar_month</span>
                    <span>{{ eventStartDateTimeLabel }}</span>
                  </span>
                </div>
              </div>
              <div v-if="hasEventAdminAccess || (headerJoinRoute && event.status === 'ACTIVE')" class="event-header-actions">
                <AppButton
                  v-if="hasEventAdminAccess"
                  :disabled="updatingFeaturedEvent"
                  :full-width="false"
                  :with-top-spacing="false"
                  @click="setFeaturedEvent(!event.is_featured)"
                >
                  {{ updatingFeaturedEvent ? t('eventPage.updatingFeatured') : (event.is_featured ? t('eventPage.removeSpotlight') : t('eventPage.setSpotlight')) }}
                </AppButton>
                <ActionCtaButton v-if="headerJoinRoute && event.status === 'ACTIVE'" :to="headerJoinRoute">
                  <span class="material-symbols-rounded" aria-hidden="true">how_to_reg</span>
                  {{ t('eventPage.joinEvent') }}
                </ActionCtaButton>
              </div>
            </div>
          </div>

          <section class="event-panel">
            <OverviewSection v-if="activeSection === 'overview'" />
            <RosterSection v-else-if="activeSection === 'roster'" />
            <TeamsSection v-else-if="activeSection === 'teams'" />
            <MatchesSection v-else-if="activeSection === 'matches'" />
            <SignupRequestsSection v-else-if="activeSection === 'requests' && canManageEvent" />
            <SettingsSection v-else-if="activeSection === 'settings' && canManageEvent" />
            <OverviewSection v-else />
          </section>
        </section>
      </div>

    </section>

    <section v-else class="event-not-found-state">
      <h2>{{ t('eventPage.notFound') }}</h2>
      <p class="muted">{{ t('eventPage.notFoundHint') }}</p>
      <button class="btn-secondary" @click="navigateToHome">{{ t('eventPage.backToEvents') }}</button>
    </section>
  </main>
</template>

<style scoped>
.event-header-card {
  padding: 1.35rem 1.15rem;
}

.event-header-top {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  min-width: 0;
}

.event-shell {
  padding: 1.1rem 1.2rem 1.25rem;
}

.event-workspace-card {
  display: flex;
  flex-direction: column;
}

.event-loading-state,
.event-not-found-state {
  padding: 0.2rem 0;
}

.event-title-name-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.event-title-row {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  gap: 0.5rem;
  min-width: 0;
  flex: 1;
}

.event-title-row h2 {
  margin: 0;
  font-size: clamp(1.35rem, 1.3vw + 1rem, 2.4rem);
  line-height: 1.1;
}

.event-starts-in {
  margin: 0;
  font-size: 0.82rem;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 0.8rem;
  flex-wrap: wrap;
}

.event-start-meta {
  display: inline-flex;
  align-items: center;
  gap: 0.22rem;
}

.event-start-separator {
  color: var(--ink-muted);
  opacity: 0.75;
}

.event-start-meta .material-symbols-rounded {
  font-size: 0.92rem;
  color: var(--ink-muted);
}

.event-logo {
  width: 4.2rem;
  height: 4.2rem;
  flex-shrink: 0;
  border-radius: var(--radius-md);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid color-mix(in srgb, var(--brand-1) 72%, #ffd869 28%);
  background: color-mix(in srgb, var(--brand-1) 14%, transparent 86%);
  box-shadow: none;
  padding: 0.4rem;
}

.event-logo-icon {
  font-size: 2.5rem;
  line-height: 1;
  color: color-mix(in srgb, var(--brand-1) 90%, #ffd869 10%);
}

.event-header-actions {
  display: flex;
  gap: 0.35rem;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  flex-shrink: 0;
  margin-left: auto;
}

.event-layout {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  gap: 0.75rem;
  align-items: start;
  margin-bottom: 0;
}

.event-main-column {
  display: grid;
  gap: 1.2rem;
  min-width: 0;
}

.event-left-nav {
  position: sticky;
  top: 5.1rem;
  display: grid;
  gap: 0.34rem;
  border: 0;
  border-radius: 0;
  padding: 0;
  background: transparent;
  box-shadow: none;
  align-self: start;
  height: fit-content;
}

.event-left-nav-kicker {
  margin: 0 0 0.2rem;
  padding-left: calc(0.62rem + 1.45rem + 0.38rem);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--ink-muted);
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
  border: 0;
  background: transparent;
  color: var(--ink-2);
  border-radius: var(--radius-md);
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
  border-radius: var(--radius-sm);
  border: 0;
  background: transparent;
  color: currentColor;
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
  background: color-mix(in srgb, var(--brand-1) 7%, transparent 93%);
  transform: translateX(2px);
}

.left-nav-item:hover .left-nav-icon {
  background: transparent;
  color: color-mix(in srgb, var(--ink-1) 88%, var(--brand-1) 12%);
  transform: translateY(-1px);
}

.left-nav-item:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--brand-2) 50%, white 50%);
  outline-offset: 1px;
}

.left-nav-item.active {
  background: color-mix(in srgb, var(--brand-1) 14%, transparent 86%);
  color: color-mix(in srgb, var(--brand-1) 88%, var(--ink-1) 12%);
  box-shadow: none;
}

.left-nav-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 3px;
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--brand-1) 84%, #fff 16%);
}

.left-nav-item.active .left-nav-icon {
  background: transparent;
  color: color-mix(in srgb, var(--brand-1) 92%, #ffe08f 8%);
}

.left-nav-badge {
  min-width: 1.35rem;
  height: 1.35rem;
  padding: 0 0.35rem;
  border-radius: var(--radius-pill);
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
  border: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
  animation: rise-in 360ms ease-out;
  padding: 0;
  display: grid;
  gap: 1.25rem;
  min-width: 0;
}

.event-shell :deep(.card) {
  border: 1px solid color-mix(in srgb, var(--line-strong) 58%, var(--bg-0) 42%);
  background: color-mix(in srgb, var(--card) 62%, var(--bg-1) 38%);
  background-image: none;
  box-shadow: none;
  border-radius: var(--radius-md);
}

.event-panel :deep(.card) {
  padding: 1.85rem;
}

.event-bottom-nav {
  display: none;
}

@media (max-width: 900px) {
  .event-shell {
    padding: 0.75rem 0.85rem;
    padding-bottom: calc(4.5rem + env(safe-area-inset-bottom, 0px));
  }

  .event-workspace-card {
    min-height: 0;
  }

  .event-layout {
    grid-template-columns: 1fr;
  }

  .event-left-nav {
    display: none;
  }

  .event-logo {
    width: 3.4rem;
    height: 3.4rem;
  }

  .event-logo-icon {
    font-size: 1.9rem;
  }

  .event-header-top {
    flex-wrap: wrap;
  }

  .event-header-actions {
    width: 100%;
    justify-content: center;
    margin-left: 0;
  }

  .event-starts-in-countdown {
    display: none;
  }

  /* Mobile bottom tab bar */
  .event-bottom-nav {
    display: flex;
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 200;
    background: var(--bg-0, #111);
    border-top: 1px solid var(--line);
    padding: 0.35rem 0.25rem calc(0.35rem + env(safe-area-inset-bottom, 0px));
    gap: 0;
    align-items: stretch;
  }

  .bottom-nav-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.18rem;
    border: none;
    background: transparent;
    color: var(--ink-3);
    cursor: pointer;
    padding: 0.3rem 0.2rem;
    border-radius: var(--radius-sm);
    transition: color 0.14s;
    position: relative;
    min-width: 0;
  }

  .bottom-nav-item.active {
    color: color-mix(in srgb, var(--brand-1) 92%, #ffe08f 8%);
  }

  .bottom-nav-icon {
    font-size: 1.45rem;
    line-height: 1;
  }

  .bottom-nav-label {
    font-size: 0.6rem;
    font-weight: 700;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    line-height: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .bottom-nav-badge {
    position: absolute;
    top: 0.15rem;
    right: calc(50% - 0.95rem);
    min-width: 1.1rem;
    height: 1.1rem;
    padding: 0 0.25rem;
    border-radius: var(--radius-pill);
    background: #ff5a3d;
    color: white;
    font-size: 0.62rem;
    font-weight: 800;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .event-panel {
    height: auto;
  }
}
</style>
