<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { overwatchRanks } from '../lib/ranks'
import { formatEventStartDate } from '../lib/dates'
import { useEventStore } from '../stores/event'
import InlineArrowLink from '../components/ui/InlineArrowLink.vue'

const route = useRoute()
const router = useRouter()
const eventStore = useEventStore()

const loading = ref(false)
const submitting = ref(false)
const error = ref('')
const notice = ref('')
const signupInfo = ref(null)

const playerName = ref('')
const playerRole = ref('')
const playerRank = ref('')

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
    playerRole.value.trim().length > 0 &&
    playerRank.value.trim().length > 0 &&
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
    playerRole.value = ''
    playerRank.value = ''
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
      <InlineArrowLink class="join-back-link" to="/events" label="Return to events list" arrow-side="left" />
      <h1 class="page-title">Join Event</h1>
    </header>

    <section class="card join-card">
      <p v-if="loading" class="join-loading">Loading signup page...</p>

      <template v-else-if="signupInfo">
        <div class="join-head">
          <p class="join-eyebrow-pill">Public Signup</p>
          <h2 class="join-event-title">{{ signupInfo.event_name }}</h2>
          <div class="join-event-meta-row">
            <span class="join-event-meta-item">
              <span class="material-symbols-rounded" aria-hidden="true">trophy</span>
              <span>{{ signupInfo.event_type }}</span>
            </span>
            <span class="material-symbols-rounded join-event-meta-dot" aria-hidden="true">fiber_manual_record</span>
            <span class="join-event-meta-item join-event-meta-item-format">
              <span>{{ signupInfo.format || '5v5' }}</span>
            </span>
          </div>
          <p v-if="signupInfo.event_description" class="join-event-description">{{ signupInfo.event_description }}</p>
          <template v-if="rosterFull && !signupRequestsFull">
            <div class="join-full-state">
              <span class="material-symbols-rounded join-full-state-icon" aria-hidden="true">info</span>
              <div class="join-full-state-copy">
                <p class="join-full-state-title">Event is currently full</p>
                <p class="join-full-state-text">You can still send a request to join. The tournament organizer may increase slots or approve pending requests manually.</p>
              </div>
            </div>
            <div class="join-separator" aria-hidden="true"></div>
          </template>
          <div class="join-stats">
            <article class="join-stat-card">
              <span class="join-stat-icon-wrap" aria-hidden="true">
                <span class="material-symbols-rounded join-stat-icon">groups</span>
              </span>
              <div class="join-stat-copy">
                <span class="join-stat-label">Registered players</span>
                <strong class="join-stat-value">
                  {{ signupInfo.current_players }}/{{ signupInfo.max_players }}
                  <span v-if="rosterFull">(Full)</span>
                </strong>
              </div>
            </article>
            <article class="join-stat-card">
              <span class="join-stat-icon-wrap" aria-hidden="true">
                <span class="material-symbols-rounded join-stat-icon">calendar_month</span>
              </span>
              <div class="join-stat-copy">
                <span class="join-stat-label">{{ signupInfo.event_type === 'TOURNEY' ? 'Tournament start' : 'Event start' }}</span>
                <strong class="join-stat-value">{{ formattedStartDate }}</strong>
              </div>
            </article>
          </div>
        </div>

        <p v-if="error" class="status status-error">{{ error }}</p>
        <p v-else-if="notice" class="status status-ok">{{ notice }}</p>

        <p v-if="signupRequestsFull" class="status status-blocked">Signup is currently unavailable because this event reached the request limit.</p>

        <form class="join-form" @submit.prevent="submitRequest">
          <label class="join-field join-field-full">
            YOUR DISPLAY NAME
            <div class="join-input-leading-icon">
              <span class="material-symbols-rounded" aria-hidden="true">sports_esports</span>
              <input v-model="playerName" placeholder="Your battletag or nickname" />
            </div>
          </label>

          <label class="join-field">
            PREFERRED ROLE
            <select v-model="playerRole">
              <option value="" disabled hidden></option>
              <option value="Tank">Tank</option>
              <option value="DPS">DPS</option>
              <option value="Support">Support</option>
            </select>
          </label>

          <label class="join-field">
            CURRENT RANK
            <select v-model="playerRank">
              <option value="" disabled hidden></option>
              <option v-for="rank in overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
            </select>
          </label>

          <div class="join-actions">
            <button type="submit" class="btn-primary" :disabled="!canSubmit">
              {{ submitting ? 'Submitting...' : 'Request to join' }}
            </button>
            <p class="join-actions-note">By requesting to join, you agree to our Tournament Fair Play Guidelines.</p>
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
  grid-template-columns: minmax(0, 1fr);
}

.page-header {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: flex-start;
  gap: 0.45rem;
  width: min(100%, 860px);
  margin-inline: auto;
}

.page-title {
  margin: 0;
  font-size: clamp(2rem, 2.8vw + 1rem, 3rem);
  font-weight: 800;
  letter-spacing: -0.02em;
  line-height: 1.05;
  width: 100%;
  text-align: center;
}

.join-back-link {
  display: inline-flex;
  align-items: center;
  gap: 0.36rem;
  color: var(--ink-2);
  text-decoration: none;
  font-weight: 600;
}

.join-back-link :deep(svg) {
  width: 0.9rem;
  height: 0.9rem;
}

.join-card {
  --join-card-pad: clamp(1.35rem, 2.6vw, 2rem);
  width: min(100%, 860px);
  margin-inline: auto;
  padding: clamp(1.35rem, 2.4vw, 2rem) var(--join-card-pad);
  display: grid;
  gap: 1rem;
  border: 1px solid var(--surface-card-border);
  background: var(--card);
  box-shadow: none;
}

.join-head {
  display: grid;
  gap: 0.4rem;
  margin-bottom: 0.25rem;
}

.join-eyebrow-pill {
  margin: 0;
  color: var(--primary-200);
  border: 1px solid color-mix(in srgb, var(--primary-200) 88%, white 12%);
  background: color-mix(in srgb, var(--primary-700) 28%, var(--card) 72%);
  width: fit-content;
  border-radius: 999px;
  padding: 0.2rem 0.54rem;
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.join-event-title {
  margin: 0;
  font-size: clamp(1.35rem, 1.8vw + 0.9rem, 1.95rem);
  font-weight: 800;
}

.join-event-meta-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.9rem;
  margin: 0.15rem 0 0.3rem;
}

.join-event-meta-item {
  display: inline-flex;
  align-items: center;
  gap: 0.34rem;
  color: var(--primary-300);
  font-weight: 700;
  font-size: 0.88rem;
}

.join-event-meta-item-format {
  gap: 0;
}

.join-event-meta-item .material-symbols-rounded {
  font-size: 1.02rem;
}

.join-event-meta-dot {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: center;
  line-height: 1;
  font-size: 0.5rem;
  color: var(--primary-300);
  transform: translateY(1px);
  font-variation-settings: 'FILL' 1, 'wght' 700, 'GRAD' 0, 'opsz' 24;
}

.join-event-description {
  margin: 0.12rem 0 0.45rem;
  max-width: 66ch;
  line-height: 1.45;
  color: var(--ink-1);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.join-stats {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.72rem;
  margin-bottom: 0.9rem;
}

.join-stat-card {
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: center;
  gap: 0.72rem;
  border: 1px solid var(--surface-card-border);
  border-radius: 18px;
  padding: 0.74rem 0.86rem;
  background: var(--surface-card-bg);
  color: var(--ink-1);
}

.join-stat-icon-wrap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 2.8rem;
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--line-strong) 74%, var(--line) 26%);
  background: color-mix(in srgb, var(--bg-0) 84%, black 16%);
}

.join-stat-icon {
  font-size: 1.22rem;
  color: var(--primary-300);
}

.join-stat-copy {
  display: grid;
  gap: 0.12rem;
}

.join-stat-label {
  font-size: 0.7rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: color-mix(in srgb, var(--ink-2) 88%, white 12%);
  font-weight: 700;
}

.join-stat-value {
  font-size: 1.02rem;
  letter-spacing: 0;
  line-height: 1.2;
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

.join-full-state {
  width: 100%;
  margin: 0.75rem 0;
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: start;
  gap: 0.56rem;
  border: 1px solid color-mix(in srgb, var(--line-strong) 78%, var(--line) 22%);
  border-radius: var(--radius-sm);
  background: transparent;
  padding: 0.95rem 0.72rem;
}

.join-full-state-icon {
  margin-top: 0.03rem;
  font-size: 1.3rem;
  color: var(--ink-1);
}

.join-full-state-copy {
  display: grid;
  gap: 0.14rem;
  min-width: 0;
}

.join-full-state-title {
  margin: 0;
  color: white;
  font-size: 0.98rem;
  font-weight: 800;
  line-height: 1.2;
}

.join-full-state-text {
  margin: 0;
  color: var(--ink-1);
  line-height: 1.4;
}

.join-separator {
  height: 1px;
  width: calc(100% + (var(--join-card-pad) * 2));
  margin: 1.05rem 0 1.05rem calc(var(--join-card-pad) * -1);
  background: color-mix(in srgb, var(--line) 72%, transparent);
}

.join-form {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
}

.join-field {
  display: grid;
  gap: 0.32rem;
}

.join-input-leading-icon {
  position: relative;
  display: flex;
  align-items: center;
}

.join-input-leading-icon .material-symbols-rounded {
  position: absolute;
  left: 0.72rem;
  font-size: 1rem;
  color: var(--ink-muted);
  pointer-events: none;
}

.join-input-leading-icon input {
  width: 100%;
  padding-left: 2.15rem;
}

.join-form input::placeholder,
.join-form textarea::placeholder {
  color: color-mix(in srgb, var(--ink-muted) 70%, var(--bg-0) 30%);
}

.join-form :is(input, select, textarea) {
  background: var(--card);
}

.join-field-full {
  grid-column: 1 / -1;
}

.join-actions {
  grid-column: 1 / -1;
  display: block;
}

.join-actions .btn-primary {
  width: 100%;
  min-width: 0;
  font-weight: 800;
}

.join-actions-note {
  margin: 0.9rem 0 1rem;
  text-align: center;
  color: white;
  font-size: 0.76rem;
  line-height: 1.35;
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
  .join-stats {
    grid-template-columns: 1fr;
  }

  .join-form {
    grid-template-columns: 1fr;
  }

  .join-actions .btn-primary {
    width: 100%;
    min-width: 0;
  }
}
</style>
