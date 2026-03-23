<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const error = ref('')

onMounted(async () => {
  const query = route.query as Record<string, string>

  if (query.error) {
    const oauthError = query.error
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

  if (query.connected === 'true') {
    try {
      await authStore.fetchMe()
    } catch {
      // best effort — user may not be authenticated in this tab
    }
    const profileId = query.profile_id || authStore.user?.id
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
    const redirect = typeof query.redirect === 'string' ? query.redirect : '/events'
    router.replace(redirect)
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
