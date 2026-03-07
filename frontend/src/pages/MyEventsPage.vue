<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { apiCall } from '../lib/api'
import EventListItem from '../components/events/EventListItem.vue'

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
        <EventListItem
          v-for="event in ownedEvents"
          :key="event.id"
          :event="event"
          :show-creator="true"
          @select="openEvent(event.id)"
        />
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

.empty-state {
  display: grid;
  gap: 0.6rem;
  justify-items: start;
}
</style>
