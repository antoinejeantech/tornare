<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { apiCall } from '../lib/api'

const router = useRouter()

const events = ref([])
const loading = ref(false)
const error = ref('')

const ownedEvents = computed(() => events.value.filter((event) => Boolean(event.is_owner)))

function setError(message) {
  error.value = message
}

async function loadMyEvents() {
  loading.value = true
  try {
    error.value = ''
    events.value = await apiCall('/api/events')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to load your events')
  } finally {
    loading.value = false
  }
}

function openEvent(eventId) {
  router.push({ name: 'event', params: { id: eventId } })
}

function goToAllEvents() {
  router.push({ name: 'events' })
}

onMounted(loadMyEvents)
</script>

<template>
  <main class="app-shell">
    <header class="page-header">
      <h1 class="page-title">My Events</h1>
    </header>

    <section class="card">
      <p class="muted">Events where you are the owner. You can manage settings, roster, teams, and matchups there.</p>
    </section>

    <section class="card">
      <p v-if="loading">Loading your events...</p>
      <p v-else-if="error" class="status status-error">{{ error }}</p>
      <div v-else-if="ownedEvents.length === 0" class="empty-state">
        <p class="muted">You do not own any events yet.</p>
        <button class="btn-primary" @click="goToAllEvents">Create your first event</button>
      </div>
      <ul v-else class="my-events-list">
        <li v-for="event in ownedEvents" :key="event.id" class="my-event-row">
          <button class="my-event-select" @click="openEvent(event.id)">
            <span class="my-event-title">{{ event.name }}</span>
            <span class="muted">{{ event.event_type }} · by {{ event.creator_name || 'Unknown' }} · {{ event.matches.length }} matches · {{ event.max_players }} players</span>
          </button>
        </li>
      </ul>
    </section>
  </main>
</template>

<style scoped>
.my-events-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.my-event-row {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-1) 8%);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
  border-radius: 10px;
  padding: 0.64rem 0.7rem;
}

.my-event-select {
  all: unset;
  display: grid;
  gap: 0.2rem;
  min-width: 0;
  width: 100%;
  cursor: pointer;
}

.my-event-select:hover .my-event-title {
  color: var(--brand-1);
}

.my-event-title {
  font-weight: 800;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.empty-state {
  display: grid;
  gap: 0.6rem;
  justify-items: start;
}
</style>
