<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { apiCall } from '../lib/api'
import { useAuthStore } from '../stores/auth'
import overwatchLogo from '../assets/ranks/overwatch-logo.png'
import { formatEventStartDate } from '../lib/dates'

const router = useRouter()
const authStore = useAuthStore()

const events = ref([])
const error = ref('')
const notice = ref('')
const loadingEvents = ref(false)
const creatingEvent = ref(false)
const deletingEventId = ref(null)

const newEventName = ref('')
const newEventDescription = ref('')
const newEventStartDate = ref('')
const newEventType = ref('PUG')
const newEventMaxPlayers = ref(10)

const canCreateEvent = computed(() => {
  if (!authStore.isAuthenticated) {
    return false
  }

  return (
    newEventName.value.trim().length > 0 &&
    newEventDescription.value.trim().length <= 5000 &&
    Number.isInteger(Number(newEventMaxPlayers.value)) &&
    Number(newEventMaxPlayers.value) >= 2 &&
    Number(newEventMaxPlayers.value) <= 12
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

function eventStartLabel(event) {
  const formatted = formatEventStartDate(event?.start_date)
  return formatted || ''
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
        max_players: Number(newEventMaxPlayers.value)
      })
    })

    events.value.unshift(created)
    newEventName.value = ''
    newEventDescription.value = ''
    newEventStartDate.value = ''
    newEventType.value = 'PUG'
    newEventMaxPlayers.value = 10
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

onMounted(loadEvents)
</script>

<template>
  <main class="app-shell">
    <header class="page-header">
      <h1 class="page-title">Overwatch Event Manager</h1>
    </header>

    <p v-if="error" class="status status-error">{{ error }}</p>
    <p v-else-if="notice" class="status status-ok">{{ notice }}</p>

    <section v-if="authStore.isAuthenticated" class="card">
      <h2>Create event</h2>
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
          Max players
          <input v-model.number="newEventMaxPlayers" min="2" max="12" type="number" />
        </label>
        <button type="submit" class="btn-primary" :disabled="!canCreateEvent || creatingEvent">
          {{ creatingEvent ? 'Creating...' : 'Create event' }}
        </button>
      </form>
    </section>

    <section v-else class="card">
      <h2>Create event</h2>
      <p class="muted">Sign in to create and manage your own events.</p>
    </section>

    <section class="card">
      <h2>Events</h2>
      <p v-if="loadingEvents">Loading events...</p>
      <p v-else-if="events.length === 0" class="muted">No events yet. Create your first one above.</p>
      <ul v-else class="home-events-list">
        <li v-for="event in events" :key="event.id" class="home-event-row">
          <button class="home-event-select" @click="openEvent(event.id)">
            <span class="event-title-wrap">
              <img class="overwatch-logo" :src="overwatchLogo" alt="Overwatch logo" />
              <span class="home-event-title">{{ event.name }}</span>
            </span>
            <span class="muted">{{ event.event_type }} · by {{ event.creator_name || 'Unknown' }}<template v-if="eventStartLabel(event)"> · {{ eventStartLabel(event) }}</template> · {{ event.players.length }}/{{ event.max_players }} players</span>
          </button>
          <button
            v-if="authStore.isAuthenticated"
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
        </li>
      </ul>
    </section>
  </main>
</template>

<style scoped>
.home-events-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.home-event-row {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-1) 8%);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
  border-radius: 10px;
  padding: 0.64rem 0.7rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.7rem;
}

.home-event-select {
  all: unset;
  display: grid;
  gap: 0.2rem;
  min-width: 0;
  flex: 1;
  cursor: pointer;
}

.home-event-select:hover .home-event-title {
  color: var(--brand-1);
}

.home-event-title {
  font-weight: 800;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.event-title-wrap {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
}

.overwatch-logo {
  width: 18px;
  height: 18px;
  object-fit: contain;
  flex: 0 0 auto;
}
</style>
