<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const error = ref('')

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

onMounted(async () => {
  const oauthError = getQueryParam(route.query.error)
  const connected = getQueryParam(route.query.connected)
  const profileIdQuery = getQueryParam(route.query.profile_id)
  const redirectQuery = getQueryParam(route.query.redirect)

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

  // Tokens are passed via URL fragment to avoid exposure in logs and history.
  const hash = window.location.hash.slice(1)
  const hashParams = new URLSearchParams(hash)
  const access_token = hashParams.get('access_token')
  const refresh_token = hashParams.get('refresh_token')

  // Clear the fragment from browser history before doing anything with the tokens.
  history.replaceState(null, '', window.location.pathname + window.location.search)

  if (!access_token || !refresh_token) {
    error.value = 'Invalid callback parameters.'
    return
  }

  try {
    await authStore.initFromOAuth(access_token, refresh_token)
    router.replace(sanitizeRedirectPath(redirectQuery || '/events'))
  } catch {
    error.value = 'Authentication failed. Please try again.'
  }
})
</script>

<template>
  <main class="app-shell auth-shell">
    <section class="card auth-card">
      <template v-if="error">
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
</style>
