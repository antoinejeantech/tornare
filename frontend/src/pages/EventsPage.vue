<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { apiCall } from '../lib/api'
import { useAuthStore } from '../stores/auth'
import { formatOptionsForType } from '../lib/event-format'
import EventListItem from '../components/events/EventListItem.vue'

const router = useRouter()
const authStore = useAuthStore()

const events = ref([])
const error = ref('')
const notice = ref('')
const loadingEvents = ref(false)
const creatingEvent = ref(false)
const deletingEventId = ref(null)
const activeEventsFilter = ref('all')

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

watch(newEventType, () => {
  if (!isSelectedFormatValid.value) {
    newEventFormat.value = availableFormatOptions.value[0]
  }
})

const filteredEvents = computed(() => {
  if (activeEventsFilter.value === 'mine') {
    return events.value.filter((event) => Boolean(event.is_owner))
  }

  return events.value
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

function setEventsFilter(filter) {
  activeEventsFilter.value = filter
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
        format: newEventFormat.value,
        max_players: Number(newEventMaxPlayers.value)
      })
    })

    events.value.unshift(created)
    newEventName.value = ''
    newEventDescription.value = ''
    newEventStartDate.value = ''
    newEventType.value = 'PUG'
    newEventFormat.value = '5v5'
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

    <nav v-if="authStore.isAuthenticated" class="events-subnav" aria-label="Event filters">
      <button
        class="events-subnav-btn"
        :class="{ active: activeEventsFilter === 'all' }"
        :aria-pressed="activeEventsFilter === 'all'"
        @click="setEventsFilter('all')"
      >
        All Events
      </button>
      <button
        class="events-subnav-btn"
        :class="{ active: activeEventsFilter === 'mine' }"
        :aria-pressed="activeEventsFilter === 'mine'"
        @click="setEventsFilter('mine')"
      >
        My Events
      </button>
    </nav>

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
      <p v-else-if="filteredEvents.length === 0" class="muted">
        {{ activeEventsFilter === 'mine' ? 'You do not own any events yet.' : 'No events yet. Create your first one above.' }}
      </p>
      <ul v-else class="home-events-list">
        <EventListItem
          v-for="event in filteredEvents"
          :key="event.id"
          :event="event"
          :show-creator="true"
          @select="openEvent(event.id)"
        >
          <template #actions>
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
          </template>
        </EventListItem>
      </ul>
    </section>
  </main>
</template>

<style scoped>
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

.home-events-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

</style>
