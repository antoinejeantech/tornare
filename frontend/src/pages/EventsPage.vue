<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { apiCall } from '../lib/api'
import { useAuthStore } from '../stores/auth'
import { formatOptionsForType } from '../lib/event-format'
import EventListItem from '../components/events/EventListItem.vue'
import SpotlightEventCard from '../components/events/SpotlightEventCard.vue'

const authStore = useAuthStore()

const events = ref([])
const error = ref('')
const notice = ref('')
const loadingEvents = ref(false)
const creatingEvent = ref(false)
const activeOwnerFilter = ref('all')
const activeTypeFilter = ref('all')
const eventSearchQuery = ref('')
const activeSort = ref('soonest')
const showCreateModal = ref(false)

const newEventName = ref('')
const newEventDescription = ref('')
const newEventStartDate = ref('')
const newEventType = ref('PUG')
const newEventFormat = ref('5v5')
const newEventSignupVisibility = ref('private')
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

  if (activeOwnerFilter.value === 'mine') {
    next = next.filter((event) => Boolean(event?.is_owner))
  }

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
  const manuallyFeatured = sortedEvents.value.find((event) => Boolean(event?.is_featured))
  if (manuallyFeatured) {
    return manuallyFeatured
  }

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
  return (
    activeOwnerFilter.value !== 'all' ||
    activeTypeFilter.value !== 'all' ||
    normalizedSearchQuery.value.length > 0 ||
    activeSort.value !== 'soonest'
  )
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

function setOwnerFilter(filter) {
  activeOwnerFilter.value = filter
}

function clearFilters() {
  activeOwnerFilter.value = 'all'
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
  newEventSignupVisibility.value = 'private'
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
        public_signup_enabled: newEventSignupVisibility.value === 'public',
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

    <section class="card events-toolbar reveal-block reveal-1">
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

        <div v-if="authStore.isAuthenticated" class="events-subnav" aria-label="Event ownership filter">
          <button
            class="events-subnav-btn"
            :class="{ active: activeOwnerFilter === 'all' }"
            :aria-pressed="activeOwnerFilter === 'all'"
            @click="setOwnerFilter('all')"
          >
            All events
          </button>
          <button
            class="events-subnav-btn"
            :class="{ active: activeOwnerFilter === 'mine' }"
            :aria-pressed="activeOwnerFilter === 'mine'"
            @click="setOwnerFilter('mine')"
          >
            My events
          </button>
        </div>

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

    <SpotlightEventCard
      v-if="featuredEvent"
      class="reveal-block reveal-2"
      :event="featuredEvent"
      badge-label="Featured Event"
    />

    <section class="card reveal-block reveal-3">
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
          :to="{ name: 'event', params: { id: event.id } }"
          class="events-list-row"
          :style="{ animationDelay: `${index * 45}ms` }"
        />
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
            Signup visibility
            <select v-model="newEventSignupVisibility">
              <option value="private">Private (link only)</option>
              <option value="public">Public (visible join link)</option>
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
  display: grid;
  gap: 0.88rem;
}

.events-shell :is(h2, h3) {
  letter-spacing: -0.01em;
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
  background: color-mix(in srgb, var(--card) 92%, #2a2a2a 8%);
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
  background: color-mix(in srgb, var(--card) 94%, #2f2f2f 6%);
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
  background: color-mix(in srgb, var(--card) 94%, #2f2f2f 6%);
}

.events-subnav-btn {
  border: 0;
  background: transparent;
  color: var(--ink-2);
  font-weight: 620;
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
  font-weight: 680;
  background: linear-gradient(130deg, var(--brand-2), var(--brand-1));
}

.events-sort {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.1rem 0.45rem;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  border-radius: 999px;
  background: color-mix(in srgb, var(--card) 94%, #2f2f2f 6%);
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
  animation: list-rise 300ms ease-out forwards;
}

.reveal-block {
  opacity: 0;
  transform: translateY(10px);
  animation: reveal-rise 380ms ease-out forwards;
}

.reveal-1 { animation-delay: 60ms; }
.reveal-2 { animation-delay: 120ms; }
.reveal-3 { animation-delay: 180ms; }

@keyframes reveal-rise {
  to {
    opacity: 1;
    transform: translateY(0);
  }
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
