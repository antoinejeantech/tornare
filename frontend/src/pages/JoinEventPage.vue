<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { overwatchRanks } from '../lib/ranks'
import { formatEventStartDate } from '../lib/dates'
import { useEventStore } from '../stores/event'

const route = useRoute()
const router = useRouter()
const eventStore = useEventStore()

const loading = ref(false)
const submitting = ref(false)
const error = ref('')
const notice = ref('')
const signupInfo = ref(null)

const playerName = ref('')
const playerRole = ref('DPS')
const playerRank = ref('Unranked')

const signupToken = computed(() => String(route.params.token || ''))
const MAX_SIGNUP_REQUESTS_PER_EVENT = 99

const rosterFull = computed(() => {
  if (!signupInfo.value) {
    return false
  }
  return signupInfo.value.current_players >= signupInfo.value.max_players
})

const signupRequestsFull = computed(() => {
  if (!signupInfo.value) {
    return false
  }
  return signupInfo.value.current_signup_requests >= MAX_SIGNUP_REQUESTS_PER_EVENT
})

const formattedStartDate = computed(() => {
  if (!signupInfo.value?.start_date) {
    return 'TBA'
  }
  return formatEventStartDate(signupInfo.value.start_date) || 'TBA'
})

const canSubmit = computed(() => {
  return (
    signupToken.value.length > 0 &&
    playerName.value.trim().length > 0 &&
    !submitting.value &&
    !signupRequestsFull.value
  )
})

function setError(message) {
  error.value = message
  notice.value = ''
}

function setNotice(message) {
  notice.value = message
  error.value = ''
}

async function loadSignupInfo() {
  if (!signupToken.value) {
    setError('Invalid signup link')
    return
  }

  loading.value = true
  try {
    signupInfo.value = await eventStore.fetchPublicSignupInfo(signupToken.value)
  } catch (err) {
    signupInfo.value = null
    setError(err instanceof Error ? err.message : 'Failed to load signup link')
  } finally {
    loading.value = false
  }
}

async function submitRequest() {
  if (!canSubmit.value) {
    return
  }

  submitting.value = true
  try {
    await eventStore.submitPublicSignupRequest(signupToken.value, {
      name: playerName.value.trim(),
      role: playerRole.value,
      rank: playerRank.value,
    })

    const destinationEventId = signupInfo.value?.event_id
    if (destinationEventId) {
      await router.push({ name: 'event', params: { id: String(destinationEventId) } })
      return
    }

    if (signupInfo.value) {
      signupInfo.value = {
        ...signupInfo.value,
        current_signup_requests: signupInfo.value.current_signup_requests + 1,
      }
    }

    playerName.value = ''
    playerRole.value = 'DPS'
    playerRank.value = 'Unranked'
    setNotice('Request sent. The event owner will review it soon.')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to submit request')
  } finally {
    submitting.value = false
  }
}

onMounted(loadSignupInfo)
</script>

<template>
  <main class="app-shell join-shell">
    <header class="page-header">
      <h1 class="page-title">Join Event</h1>
    </header>

    <section class="card join-card">
      <p v-if="loading" class="join-loading">Loading signup page...</p>

      <template v-else-if="signupInfo">
        <div class="join-head">
          <p class="join-eyebrow">Public Signup</p>
          <h2 class="join-event-title">{{ signupInfo.event_name }}</h2>
          <p class="muted join-event-meta">{{ signupInfo.event_type }} · {{ signupInfo.format || '5v5' }}</p>
          <p v-if="signupInfo.event_description" class="muted join-event-description">{{ signupInfo.event_description }}</p>
          <div class="join-stats">
            <span class="join-stat-pill">
              <span class="join-pill-label">Players</span>
              <strong class="join-pill-value">{{ signupInfo.current_players }}/{{ signupInfo.max_players }}</strong>
            </span>
            <span class="join-stat-pill">
              <span class="join-pill-label">Start</span>
              <strong class="join-pill-value">{{ formattedStartDate }}</strong>
            </span>
          </div>
        </div>

        <p v-if="error" class="status status-error">{{ error }}</p>
        <p v-else-if="notice" class="status status-ok">{{ notice }}</p>

        <p v-if="signupRequestsFull" class="status status-blocked">Signup is currently unavailable because this event reached the request limit.</p>
        <p v-else-if="rosterFull" class="status status-blocked status-blocked-soft">Event is currently full, but you can still send a request while the owner adjusts slots.</p>

        <form class="join-form" @submit.prevent="submitRequest">
          <label class="join-field join-field-full">
            Your name
            <input v-model="playerName" placeholder="Your battletag or nickname" />
          </label>

          <label class="join-field">
            Role
            <select v-model="playerRole">
              <option>Tank</option>
              <option>DPS</option>
              <option>Support</option>
            </select>
          </label>

          <label class="join-field">
            Rank
            <select v-model="playerRank">
              <option v-for="rank in overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
            </select>
          </label>

          <div class="join-actions">
            <button type="submit" class="btn-primary" :disabled="!canSubmit">
              {{ submitting ? 'Submitting...' : 'Request to join' }}
            </button>
          </div>
        </form>
      </template>

      <template v-else>
        <div class="join-unavailable">
          <h2>Signup link unavailable</h2>
          <p class="muted">This link may be invalid or has expired.</p>
        </div>
      </template>
    </section>
  </main>
</template>

<style scoped>
.join-shell {
  width: min(95vw, 940px);
}

.page-header {
  justify-content: center;
}

.page-title {
  text-align: center;
}

.join-card {
  max-width: 860px;
  margin: 0 auto;
  padding: clamp(1rem, 1.4vw, 1.3rem);
  display: grid;
  gap: 0.95rem;
  border-color: color-mix(in srgb, var(--brand-2) 30%, var(--line) 70%);
  background:
    radial-gradient(560px 220px at 100% 0%, color-mix(in srgb, var(--brand-1) 18%, transparent) 0%, transparent 72%),
    linear-gradient(160deg, color-mix(in srgb, var(--card) 92%, #f0f5ff 8%) 0%, var(--card) 100%);
}

.join-head {
  display: grid;
  gap: 0.28rem;
}

.join-eyebrow {
  margin: 0;
  color: var(--accent);
  font-size: 0.8rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.join-event-title {
  margin: 0;
  font-size: clamp(1.2rem, 1.5vw + 0.8rem, 1.65rem);
}

.join-event-meta {
  margin: 0;
}

.join-event-description {
  margin: 0.05rem 0 0.15rem;
  max-width: 66ch;
  line-height: 1.45;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.join-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.join-stat-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.34rem;
  border: 1px solid color-mix(in srgb, var(--brand-2) 35%, var(--line) 65%);
  border-radius: 999px;
  padding: 0.34rem 0.62rem;
  background: color-mix(in srgb, var(--brand-1) 10%, var(--card) 90%);
  color: var(--ink-1);
}

.join-pill-label {
  font-size: 0.78rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--ink-2);
}

.join-pill-value {
  font-size: 0.95rem;
  letter-spacing: 0.01em;
}

.status-blocked {
  color: var(--ink-1);
  background: color-mix(in srgb, var(--meta-bg) 28%, var(--card) 72%);
  border-color: color-mix(in srgb, var(--meta-ink) 26%, var(--line) 74%);
}

.status-blocked-soft {
  color: color-mix(in srgb, var(--warn) 54%, var(--ink-1) 46%);
  background: color-mix(in srgb, var(--warn) 12%, var(--card) 88%);
  border-color: color-mix(in srgb, var(--warn) 36%, var(--line) 64%);
}

.join-form {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.72rem;
}

.join-field {
  display: grid;
  gap: 0.32rem;
}

.join-field-full {
  grid-column: 1 / -1;
}

.join-actions {
  grid-column: 1 / -1;
  display: flex;
  justify-content: center;
}

.join-actions .btn-primary {
  min-width: 220px;
}

.join-loading,
.join-unavailable {
  margin: 0;
}

.join-unavailable {
  display: grid;
  gap: 0.45rem;
}

.join-unavailable h2,
.join-unavailable p {
  margin: 0;
}

@media (max-width: 700px) {
  .join-form {
    grid-template-columns: 1fr;
  }

  .join-actions .btn-primary {
    width: 100%;
    min-width: 0;
  }
}
</style>
