<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { apiCall } from '../lib/api'
import { useAuthStore } from '../stores/auth'
import { formatOptionsForType } from '../lib/event-format'
import { formatEventStartDate } from '../lib/dates'
import EventListItem from '../components/events/EventListItem.vue'

const router = useRouter()
const authStore = useAuthStore()

const events = ref([])
const error = ref('')
const notice = ref('')
const loadingEvents = ref(false)
const creatingEvent = ref(false)
const deletingEventId = ref(null)
const activeTypeFilter = ref('all')
const eventSearchQuery = ref('')
const activeSort = ref('soonest')
const showCreateModal = ref(false)

const newEventName = ref('')
const newEventDescription = ref('')
const newEventStartDate = ref('')
const newEventType = ref('PUG')
const newEventFormat = ref('5v5')
const newEventMaxPlayers = ref(10)

const availableFormatOptions = computed(() => {
  return formatOptionsForType(newEventType.value)
})

const isSelectedFormatValid = computed(() => {
  return availableFormatOptions.value.includes(newEventFormat.value)
})

const canCreateEvent = computed(() => {
  if (!authStore.isAuthenticated) {
    return false
  }

  return (
    newEventName.value.trim().length > 0 &&
    newEventDescription.value.trim().length <= 5000 &&
    Number.isInteger(Number(newEventMaxPlayers.value)) &&
    Number(newEventMaxPlayers.value) >= 2 &&
    Number(newEventMaxPlayers.value) <= 99 &&
    isSelectedFormatValid.value
  )
})

const normalizedSearchQuery = computed(() => eventSearchQuery.value.trim().toLowerCase())

watch(newEventType, () => {
  if (!isSelectedFormatValid.value) {
    newEventFormat.value = availableFormatOptions.value[0]
  }
})

const filteredEvents = computed(() => {
  let next = events.value

  if (activeTypeFilter.value !== 'all') {
    next = next.filter((event) => String(event.event_type || '').toUpperCase() === activeTypeFilter.value)
  }

  if (normalizedSearchQuery.value) {
    next = next.filter((event) => {
      const name = String(event.name || '').toLowerCase()
      const description = String(event.description || '').toLowerCase()
      const creator = String(event.creator_name || '').toLowerCase()
      return (
        name.includes(normalizedSearchQuery.value) ||
        description.includes(normalizedSearchQuery.value) ||
        creator.includes(normalizedSearchQuery.value)
      )
    })
  }

  return next
})

const sortedEvents = computed(() => {
  const next = [...filteredEvents.value]

  const sortBySoonest = (a, b) => {
    const aStart = normalizeDateValue(a?.start_date)
    const bStart = normalizeDateValue(b?.start_date)

    if (aStart === null && bStart === null) {
      return String(a?.name || '').localeCompare(String(b?.name || ''))
    }
    if (aStart === null) {
      return 1
    }
    if (bStart === null) {
      return -1
    }

    return aStart - bStart
  }

  if (activeSort.value === 'newest') {
    next.sort((a, b) => sortBySoonest(b, a))
    return next
  }

  if (activeSort.value === 'players') {
    next.sort((a, b) => getPlayerCount(b) - getPlayerCount(a))
    return next
  }

  if (activeSort.value === 'name') {
    next.sort((a, b) => String(a?.name || '').localeCompare(String(b?.name || '')))
    return next
  }

  next.sort(sortBySoonest)
  return next
})

const totalEventsCount = computed(() => events.value.length)

const totalPlayersSignedUp = computed(() => {
  return events.value.reduce((sum, event) => sum + getPlayerCount(event), 0)
})

const weeklyTourneyCount = computed(() => {
  const now = Date.now()
  const weekEnd = now + 7 * 24 * 60 * 60 * 1000

  return events.value.filter((event) => {
    const startAt = normalizeDateValue(event?.start_date)
    const isTourney = String(event?.event_type || '').toUpperCase() === 'TOURNEY'
    if (!isTourney || startAt === null) {
      return false
    }

    return startAt >= now && startAt <= weekEnd
  }).length
})

const featuredEvent = computed(() => {
  const upcoming = sortedEvents.value.find((event) => {
    const startAt = normalizeDateValue(event?.start_date)
    return startAt !== null && startAt >= Date.now()
  })

  if (upcoming) {
    return upcoming
  }

  return sortedEvents.value[0] || null
})

const hasActiveFilters = computed(() => {
  return activeTypeFilter.value !== 'all' || normalizedSearchQuery.value.length > 0 || activeSort.value !== 'soonest'
})

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

function setTypeFilter(filter) {
  activeTypeFilter.value = filter
}

function clearFilters() {
  activeTypeFilter.value = 'all'
  eventSearchQuery.value = ''
  activeSort.value = 'soonest'
}

function resetCreateForm() {
  newEventName.value = ''
  newEventDescription.value = ''
  newEventStartDate.value = ''
  newEventType.value = 'PUG'
  newEventFormat.value = '5v5'
  newEventMaxPlayers.value = 10
}

function openCreateModal() {
  if (!authStore.isAuthenticated) {
    setError('Sign in to create an event')
    return
  }

  clearError()
  clearNotice()
  showCreateModal.value = true
}

function closeCreateModal() {
  if (creatingEvent.value) {
    return
  }

  showCreateModal.value = false
}

async function loadEvents() {
  loadingEvents.value = true
  try {
    clearError()
    clearNotice()
    events.value = await apiCall('/api/events')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to load events')
  } finally {
    loadingEvents.value = false
  }
}

function openEvent(eventId) {
  clearError()
  clearNotice()
  router.push({ name: 'event', params: { id: eventId } })
}

function normalizeDateValue(value) {
  if (!value) {
    return null
  }

  const parsed = new Date(value).getTime()
  return Number.isNaN(parsed) ? null : parsed
}

function getPlayerCount(event) {
  return Array.isArray(event?.players) ? event.players.length : 0
}

function getEventStatus(event) {
  const maxPlayers = Number(event?.max_players) || 0
  const playerCount = getPlayerCount(event)
  const startAt = normalizeDateValue(event?.start_date)

  if (maxPlayers > 0 && playerCount >= maxPlayers) {
    return 'Full'
  }

  if (startAt !== null) {
    const now = Date.now()
    if (startAt <= now) {
      return 'Ongoing'
    }

    const diff = startAt - now
    if (diff <= 6 * 60 * 60 * 1000) {
      return 'Starting Soon'
    }
  }

  return 'Open'
}

function eventStatusClass(event) {
  const status = getEventStatus(event)
  if (status === 'Full') {
    return 'is-full'
  }
  if (status === 'Ongoing') {
    return 'is-ongoing'
  }
  if (status === 'Starting Soon') {
    return 'is-soon'
  }

  return 'is-open'
}

function featuredMeta(event) {
  const startText = formatEventStartDate(event?.start_date)
  const parts = [
    String(event?.event_type || 'PUG'),
    String(event?.format || '5v5'),
    `${getPlayerCount(event)}/${Number(event?.max_players) || 0} players`,
  ]

  if (startText) {
    parts.push(startText)
  }

  return parts.join(' · ')
}

function handleGlobalKeyDown(event) {
  if (event.key === 'Escape' && showCreateModal.value) {
    closeCreateModal()
  }
}

async function createEvent() {
  if (!canCreateEvent.value || creatingEvent.value) {
    return
  }

  creatingEvent.value = true
  try {
    clearError()
    clearNotice()

    const created = await apiCall('/api/events', {
      method: 'POST',
      body: JSON.stringify({
        name: newEventName.value.trim(),
        description: newEventDescription.value.trim(),
        start_date: newEventStartDate.value ? newEventStartDate.value : null,
        event_type: newEventType.value,
        format: newEventFormat.value,
        max_players: Number(newEventMaxPlayers.value)
      })
    })

    events.value.unshift(created)
    resetCreateForm()
    showCreateModal.value = false
    setNotice('Event created successfully')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to create event')
  } finally {
    creatingEvent.value = false
  }
}

async function deleteEvent(eventId) {
  if (deletingEventId.value) {
    return
  }

  const target = events.value.find((event) => event.id === eventId)
  const confirmed = window.confirm(`Delete event "${target?.name || eventId}"? This also deletes its matches.`)
  if (!confirmed) {
    return
  }

  deletingEventId.value = eventId
  try {
    clearError()
    clearNotice()

    await apiCall(`/api/events/${eventId}`, {
      method: 'DELETE'
    })

    events.value = events.value.filter((event) => event.id !== eventId)
    setNotice('Event deleted')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to delete event')
  } finally {
    deletingEventId.value = null
  }
}

onMounted(() => {
  loadEvents()
  window.addEventListener('keydown', handleGlobalKeyDown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleGlobalKeyDown)
})
</script>

<template>
  <main class="app-shell events-shell">
    <header class="page-header">
      <h1 class="page-title">Find Your Next Overwatch Event</h1>
    </header>

    <section class="card events-toolbar">
      <div class="events-toolbar-top">
        <div class="events-toolbar-title-wrap">
          <h2>Events</h2>
          <p class="muted">Browse all public events and quickly narrow the list.</p>
        </div>
        <button
          class="btn-primary events-create-btn"
          :disabled="!authStore.isAuthenticated"
          :title="authStore.isAuthenticated ? 'Create a new event' : 'Sign in to create an event'"
          @click="openCreateModal"
        >
          <span class="material-symbols-rounded" aria-hidden="true">add</span>
          <span>Create event</span>
        </button>
      </div>

      <p v-if="authStore.isAuthenticated" class="muted events-my-link-row">
        Looking for your own registrations and invites?
        <RouterLink class="events-my-link" to="/my-events">My events</RouterLink>
      </p>

      <section class="events-stats-grid" aria-label="Event highlights">
        <article class="events-stat-card">
          <span class="events-stat-label">Live board</span>
          <strong class="events-stat-value">{{ totalEventsCount }}</strong>
          <span class="muted">Active listings</span>
        </article>
        <article class="events-stat-card">
          <span class="events-stat-label">Signups</span>
          <strong class="events-stat-value">{{ totalPlayersSignedUp }}</strong>
          <span class="muted">Players currently registered</span>
        </article>
        <article class="events-stat-card">
          <span class="events-stat-label">This week</span>
          <strong class="events-stat-value">{{ weeklyTourneyCount }}</strong>
          <span class="muted">Upcoming tourneys</span>
        </article>
      </section>

      <div class="events-filter-row">
        <label class="events-search">
          <span class="sr-only">Search events</span>
          <span class="material-symbols-rounded" aria-hidden="true">search</span>
          <input v-model="eventSearchQuery" type="search" placeholder="Search by name, description, creator" />
        </label>

        <div class="events-subnav" aria-label="Event type filter">
          <button
            class="events-subnav-btn"
            :class="{ active: activeTypeFilter === 'all' }"
            :aria-pressed="activeTypeFilter === 'all'"
            @click="setTypeFilter('all')"
          >
            All types
          </button>
          <button
            class="events-subnav-btn"
            :class="{ active: activeTypeFilter === 'PUG' }"
            :aria-pressed="activeTypeFilter === 'PUG'"
            @click="setTypeFilter('PUG')"
          >
            PUG
          </button>
          <button
            class="events-subnav-btn"
            :class="{ active: activeTypeFilter === 'TOURNEY' }"
            :aria-pressed="activeTypeFilter === 'TOURNEY'"
            @click="setTypeFilter('TOURNEY')"
          >
            Tourney
          </button>
        </div>

        <label class="events-sort">
          <span class="events-sort-label">Sort</span>
          <select v-model="activeSort">
            <option value="soonest">Soonest</option>
            <option value="newest">Latest</option>
            <option value="players">Most players</option>
            <option value="name">A-Z</option>
          </select>
        </label>

        <button
          type="button"
          class="btn-secondary"
          :disabled="!hasActiveFilters"
          @click="clearFilters"
        >
          Clear filters
        </button>
      </div>
    </section>

    <p v-if="error" class="status status-error">{{ error }}</p>
    <p v-else-if="notice" class="status status-ok">{{ notice }}</p>

    <section v-if="featuredEvent" class="card featured-event-card">
      <div class="featured-event-head">
        <span class="featured-badge">Featured Tonight</span>
        <span class="event-status-chip" :class="eventStatusClass(featuredEvent)">{{ getEventStatus(featuredEvent) }}</span>
      </div>
      <h2 class="featured-event-title">{{ featuredEvent.name }}</h2>
      <p class="muted featured-event-meta">{{ featuredMeta(featuredEvent) }}</p>
      <div class="featured-event-actions">
        <button type="button" class="btn-primary" @click="openEvent(featuredEvent.id)">
          Open event
        </button>
      </div>
    </section>

    <section class="card">
      <p v-if="loadingEvents">Loading events...</p>
      <div v-else-if="sortedEvents.length === 0" class="events-empty-state">
        <h2>No events match your filters</h2>
        <p class="muted">Try widening your filters or create a new event for your community.</p>
        <div class="events-empty-actions">
          <button type="button" class="btn-secondary" :disabled="!hasActiveFilters" @click="clearFilters">Clear filters</button>
          <button type="button" class="btn-primary" :disabled="!authStore.isAuthenticated" @click="openCreateModal">Create event</button>
        </div>
      </div>
      <ul v-else class="home-events-list">
        <EventListItem
          v-for="(event, index) in sortedEvents"
          :key="event.id"
          :event="event"
          :show-creator="true"
          class="events-list-row"
          :style="{ animationDelay: `${index * 45}ms` }"
          @select="openEvent(event.id)"
        >
          <template #actions>
            <button
              class="btn-secondary icon-btn"
              title="Open event"
              @click="openEvent(event.id)"
            >
              <span class="material-symbols-rounded" aria-hidden="true">open_in_new</span>
              <span class="sr-only">Open event</span>
            </button>
            <button
              v-if="event.is_owner"
              class="btn-secondary icon-btn"
              title="Manage event"
              @click="openEvent(event.id)"
            >
              <span class="material-symbols-rounded" aria-hidden="true">edit_note</span>
              <span class="sr-only">Manage event</span>
            </button>
            <button
              v-if="event.is_owner"
              class="btn-danger icon-btn"
              :disabled="deletingEventId === event.id"
              :title="deletingEventId === event.id ? 'Deleting event' : 'Delete event'"
              @click="deleteEvent(event.id)"
            >
              <span class="material-symbols-rounded" aria-hidden="true">
                {{ deletingEventId === event.id ? 'hourglass_top' : 'delete' }}
              </span>
              <span class="sr-only">{{ deletingEventId === event.id ? 'Deleting event' : 'Delete event' }}</span>
            </button>
          </template>
        </EventListItem>
      </ul>
    </section>

    <div
      v-if="showCreateModal"
      class="events-modal-backdrop"
      role="presentation"
      @click.self="closeCreateModal"
    >
      <section
        class="events-modal card"
        role="dialog"
        aria-modal="true"
        aria-labelledby="create-event-modal-title"
      >
        <header class="events-modal-header">
          <h2 id="create-event-modal-title">Create event</h2>
          <button class="btn-secondary" type="button" :disabled="creatingEvent" @click="closeCreateModal">
            Close
          </button>
        </header>
        <form class="grid-form" @submit.prevent="createEvent">
          <label>
            Event name
            <input v-model="newEventName" placeholder="Friday Night PUG" />
          </label>
          <label>
            Description
            <textarea v-model="newEventDescription" rows="4" placeholder="Rules, cashprize, check-in info..." />
          </label>
          <label>
            Start date
            <input v-model="newEventStartDate" type="datetime-local" />
          </label>
          <label>
            Event type
            <select v-model="newEventType">
              <option value="PUG">PUG</option>
              <option value="TOURNEY">TOURNEY</option>
            </select>
          </label>
          <label>
            Format
            <select v-model="newEventFormat">
              <option v-for="format in availableFormatOptions" :key="`new-event-format-${format}`" :value="format">
                {{ format }}
              </option>
            </select>
          </label>
          <label>
            Max players
            <input v-model.number="newEventMaxPlayers" min="2" max="99" type="number" />
          </label>
          <div class="events-modal-actions">
            <button type="submit" class="btn-primary" :disabled="!canCreateEvent || creatingEvent">
              {{ creatingEvent ? 'Creating...' : 'Create event' }}
            </button>
            <button type="button" class="btn-secondary" :disabled="creatingEvent" @click="closeCreateModal">
              Cancel
            </button>
          </div>
        </form>
      </section>
    </div>
  </main>
</template>

<style scoped>
.events-shell {
  max-width: 1820px;
  width: min(96vw, 1820px);
}

.events-toolbar {
  display: grid;
  gap: 0.8rem;
}

.events-toolbar-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.7rem;
}

.events-toolbar-title-wrap {
  display: grid;
  gap: 0.2rem;
}

.events-toolbar-title-wrap h2,
.events-toolbar-title-wrap p {
  margin: 0;
}

.events-create-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.3rem;
}

.events-create-btn .material-symbols-rounded {
  font-size: 1rem;
}

.events-my-link-row {
  margin: -0.25rem 0 0;
}

.events-my-link {
  margin-left: 0.35rem;
  font-weight: 700;
  text-decoration: none;
  color: var(--brand-1);
}

.events-my-link:hover {
  text-decoration: underline;
}

.events-filter-row {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  flex-wrap: wrap;
}

.events-stats-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.55rem;
}

.events-stat-card {
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  border-radius: 12px;
  padding: 0.6rem 0.7rem;
  background: color-mix(in srgb, var(--card) 90%, #edf5ff 10%);
  display: grid;
  gap: 0.2rem;
}

.events-stat-label {
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--ink-2);
}

.events-stat-value {
  font-size: 1.35rem;
  line-height: 1;
}

.events-search {
  min-width: min(100%, 300px);
  flex: 1;
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  background: color-mix(in srgb, var(--card) 92%, #f2f7ff 8%);
  padding: 0.28rem 0.55rem;
}

.events-search input {
  flex: 1;
  border: 0;
  background: transparent;
  color: var(--ink-1);
  min-width: 0;
}

.events-search input:focus {
  outline: none;
}

.events-subnav {
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
  width: fit-content;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  border-radius: 999px;
  padding: 0.22rem;
  background: color-mix(in srgb, var(--card) 92%, #edf5ff 8%);
}

.events-subnav-btn {
  border: 0;
  background: transparent;
  color: var(--ink-2);
  font-weight: 760;
  letter-spacing: 0.01em;
  padding: 0.34rem 0.72rem;
  border-radius: 999px;
  cursor: pointer;
  transition: background 0.16s ease, color 0.16s ease;
}

.events-subnav-btn:hover {
  color: var(--ink-1);
  background: color-mix(in srgb, var(--brand-2) 10%, var(--card) 90%);
}

.events-subnav-btn.active {
  color: #fff;
  background: linear-gradient(130deg, #0f4f99, var(--brand-1));
}

.events-sort {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.1rem 0.45rem;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  border-radius: 999px;
  background: color-mix(in srgb, var(--card) 92%, #edf5ff 8%);
}

.events-sort-label {
  font-size: 0.78rem;
  color: var(--ink-2);
  font-weight: 700;
}

.events-sort select {
  border: 0;
  background: transparent;
  color: var(--ink-1);
  font-weight: 700;
}

.featured-event-card {
  display: grid;
  gap: 0.45rem;
  border-color: color-mix(in srgb, var(--brand-2) 30%, var(--line) 70%);
  background:
    radial-gradient(1200px 90px at 0% 0%, rgba(66, 133, 244, 0.16), transparent 60%),
    color-mix(in srgb, var(--card) 92%, #f0f6ff 8%);
}

.featured-event-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.featured-badge {
  font-size: 0.72rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--brand-1);
}

.featured-event-title {
  margin: 0;
}

.featured-event-meta {
  margin: 0;
}

.featured-event-actions {
  display: flex;
  align-items: center;
  gap: 0.45rem;
}

.event-status-chip {
  border-radius: 999px;
  padding: 0.2rem 0.55rem;
  font-size: 0.72rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  border: 1px solid transparent;
}

.event-status-chip.is-open {
  color: #0b5a1e;
  background: #daf4e2;
  border-color: #95d9a9;
}

.event-status-chip.is-soon {
  color: #7a3b00;
  background: #ffe8c9;
  border-color: #ffc57f;
}

.event-status-chip.is-full {
  color: #7a2a0a;
  background: #ffd9ce;
  border-color: #ffad95;
}

.event-status-chip.is-ongoing {
  color: #fff;
  background: linear-gradient(130deg, #0f4f99, var(--brand-1));
  border-color: color-mix(in srgb, #0f4f99 75%, var(--brand-1) 25%);
}

.events-empty-state {
  display: grid;
  gap: 0.45rem;
}

.events-empty-state h2,
.events-empty-state p {
  margin: 0;
}

.events-empty-actions {
  display: flex;
  align-items: center;
  gap: 0.45rem;
}

.home-events-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.events-list-row {
  opacity: 0;
  transform: translateY(8px);
  animation: list-rise 260ms ease-out forwards;
}

@keyframes list-rise {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.events-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(7, 14, 30, 0.5);
  backdrop-filter: blur(3px);
  z-index: 70;
  display: grid;
  place-items: center;
  padding: 1rem;
}

.events-modal {
  width: min(760px, 100%);
  max-height: calc(100vh - 2rem);
  overflow: auto;
}

.events-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  margin-bottom: 0.75rem;
}

.events-modal-header h2 {
  margin: 0;
}

.events-modal-actions {
  display: flex;
  align-items: center;
  gap: 0.45rem;
}

@media (max-width: 840px) {
  .events-toolbar-top {
    flex-wrap: wrap;
  }

  .events-stats-grid {
    grid-template-columns: 1fr;
  }

  .events-empty-actions {
    flex-wrap: wrap;
  }

  .events-modal-actions {
    flex-wrap: wrap;
  }
}

</style>
