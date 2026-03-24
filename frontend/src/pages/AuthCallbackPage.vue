<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const error = ref('')
const needsEmail = ref(false)
const pendingToken = ref('')
const battletag = ref('')
const email = ref('')
const submitting = ref(false)

function getQueryParam(value: unknown): string {
  if (typeof value === 'string') {
    return value
  }
  if (Array.isArray(value)) {
    return typeof value[0] === 'string' ? value[0] : ''
  }
  return ''
}

function sanitizeRedirectPath(rawRedirect: string): string {
  if (!rawRedirect.startsWith('/')) {
    return '/events'
  }
  // Reject protocol-relative URLs like //attacker.example
  if (rawRedirect.startsWith('//')) {
    return '/events'
  }
  return rawRedirect
}

function clearSensitiveCallbackData(): void {
  const url = new URL(window.location.href)
  url.hash = ''
  url.searchParams.delete('needs_email')
  url.searchParams.delete('pending_token')
  url.searchParams.delete('battletag')
  history.replaceState(null, '', url.pathname + url.search)
}

onMounted(async () => {
  const hashParams = new URLSearchParams(window.location.hash.slice(1))

  const oauthError = getQueryParam(route.query.error)
  const connected = getQueryParam(route.query.connected)
  const profileIdQuery = getQueryParam(route.query.profile_id)
  const redirectQuery = getQueryParam(route.query.redirect)
  const needsEmailFlag = hashParams.get('needs_email') || getQueryParam(route.query.needs_email)
  const pendingTokenValue = hashParams.get('pending_token') || getQueryParam(route.query.pending_token)
  const battletagValue = hashParams.get('battletag') || getQueryParam(route.query.battletag)
  const accessToken = hashParams.get('access_token')
  const refreshToken = hashParams.get('refresh_token')

  if (window.location.hash || pendingTokenValue) {
    clearSensitiveCallbackData()
  }

  if (oauthError) {
    error.value =
      oauthError === 'access_denied'
        ? 'Battle.net sign-in was cancelled.'
        : oauthError === 'oauth_not_configured'
          ? 'Battle.net login is not yet configured.'
          : oauthError === 'rate_limited'
            ? 'Too many requests. Please wait a moment and try again.'
            : 'Battle.net sign-in failed. Please try again.'
    return
  }

  if (needsEmailFlag === 'true') {
    if (!pendingTokenValue) {
      error.value = 'Battle.net sign-in is missing continuation data. Please try again.'
      return
    }
    needsEmail.value = true
    pendingToken.value = pendingTokenValue
    battletag.value = battletagValue
    return
  }

  if (connected === 'true') {
    try {
      await authStore.fetchMe()
    } catch {
      // best effort — user may not be authenticated in this tab
    }
    const profileId = profileIdQuery || authStore.user?.id
    router.replace(profileId ? `/profiles/${profileId}` : '/events')
    return
  }

  if (!accessToken || !refreshToken) {
    error.value = 'Invalid callback parameters.'
    return
  }

  try {
    await authStore.initFromOAuth(accessToken, refreshToken)
    router.replace(sanitizeRedirectPath(redirectQuery || '/events'))
  } catch {
    error.value = 'Authentication failed. Please try again.'
  }
})

async function submitEmail(): Promise<void> {
  if (!needsEmail.value || submitting.value) {
    return
  }

  const trimmedEmail = email.value.trim()
  if (!trimmedEmail || !trimmedEmail.includes('@')) {
    error.value = 'Please enter a valid email address.'
    return
  }

  submitting.value = true
  error.value = ''
  try {
    await authStore.completeBnetSignup(pendingToken.value, trimmedEmail)
    router.replace('/events')
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to complete sign-up.'
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <main class="app-shell auth-shell">
    <section class="card auth-card">
      <template v-if="needsEmail">
        <h1>One more step</h1>
        <p class="muted">
          Battle.net did not provide an email for
          <strong v-if="battletag">{{ battletag }}</strong>
          <span v-else>your account</span>.
          Enter your email to complete sign-up.
        </p>
        <form class="email-form" @submit.prevent="submitEmail">
          <label class="field-label" for="oauth-email">Email</label>
          <input
            id="oauth-email"
            v-model="email"
            class="field-input"
            type="email"
            name="email"
            autocomplete="email"
            required
            :disabled="submitting"
          />
          <button class="btn-primary" type="submit" :disabled="submitting">
            {{ submitting ? 'Completing…' : 'Complete sign-up' }}
          </button>
        </form>
        <p v-if="error" class="status status-error">{{ error }}</p>
      </template>
      <template v-else-if="error">
        <p class="status status-error">{{ error }}</p>
        <RouterLink to="/auth" class="btn-secondary">Back to sign in</RouterLink>
      </template>
      <template v-else>
        <p class="muted">Signing you in with Battle.net…</p>
      </template>
    </section>
  </main>
</template>

<style scoped>
.auth-shell {
  min-height: calc(100vh - 220px);
  display: grid;
  align-content: center;
}

.auth-card {
  max-width: 560px;
  margin: 0 auto;
  display: grid;
  gap: 1.5rem;
  text-align: center;
}

.email-form {
  display: grid;
  gap: 0.75rem;
  text-align: left;
}

.field-label {
  font-weight: 700;
}

.field-input {
  width: 100%;
  padding: 0.7rem 0.85rem;
  border-radius: var(--radius-md);
  border: 1px solid var(--line);
  background: var(--input-bg);
  color: var(--input-ink);
}
</style>
