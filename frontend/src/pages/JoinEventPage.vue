<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { overwatchRanks } from '../lib/ranks'
import { useEventStore } from '../stores/event'

const route = useRoute()
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
const rosterFull = computed(() => {
  if (!signupInfo.value) {
    return false
  }
  return signupInfo.value.current_players >= signupInfo.value.max_players
})

const canSubmit = computed(() => {
  return (
    signupToken.value.length > 0 &&
    playerName.value.trim().length > 0 &&
    !submitting.value &&
    !rosterFull.value
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
  <main class="app-shell">
    <header class="page-header">
      <h1 class="page-title">Join Event</h1>
    </header>

    <section class="card join-card">
      <p v-if="loading">Loading signup page...</p>
      <template v-else-if="signupInfo">
        <h2>{{ signupInfo.event_name }}</h2>
        <p class="muted">{{ signupInfo.event_type }} · {{ signupInfo.current_players }}/{{ signupInfo.max_players }} players</p>

        <p v-if="error" class="status status-error">{{ error }}</p>
        <p v-else-if="notice" class="status status-ok">{{ notice }}</p>

        <p v-if="rosterFull" class="muted">This roster is currently full. You can try again later.</p>

        <form class="grid-form" @submit.prevent="submitRequest">
          <label>
            Your name
            <input v-model="playerName" placeholder="Your battletag or nickname" />
          </label>

          <label>
            Role
            <select v-model="playerRole">
              <option>Tank</option>
              <option>DPS</option>
              <option>Support</option>
            </select>
          </label>

          <label>
            Rank
            <select v-model="playerRank">
              <option v-for="rank in overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
            </select>
          </label>

          <button type="submit" class="btn-primary" :disabled="!canSubmit">
            {{ submitting ? 'Submitting...' : 'Request to join' }}
          </button>
        </form>
      </template>

      <template v-else>
        <h2>Signup link unavailable</h2>
        <p class="muted">This link may be invalid or has expired.</p>
      </template>
    </section>
  </main>
</template>

<style scoped>
.join-card {
  max-width: 620px;
}
</style>
