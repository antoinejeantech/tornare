<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../stores/auth'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const error = ref('')
const needsEmail = ref(false)
const pendingToken = ref('')
const battletag = ref('')
const email = ref('')
const submitting = ref(false)
const returnPath = ref('/login')

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
  const stored = sessionStorage.getItem('oauth_return_path')
  if (stored) {
    returnPath.value = sanitizeRedirectPath(stored)
    sessionStorage.removeItem('oauth_return_path')
  }

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
        ? t('authCallback.cancelled')
        : oauthError === 'oauth_not_configured'
          ? t('authCallback.notConfigured')
          : oauthError === 'rate_limited'
            ? t('authCallback.rateLimited')
            : t('authCallback.failed')
    return
  }

  if (needsEmailFlag === 'true') {
    if (!pendingTokenValue) {
      error.value = t('authCallback.missingData')
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
    const resumeStep = sessionStorage.getItem('onboarding_resume_step')
    if (resumeStep) {
      sessionStorage.removeItem('onboarding_resume_step')
      router.replace({ name: 'onboarding', query: { step: resumeStep } })
      return
    }
    const profileId = profileIdQuery || authStore.user?.id
    router.replace(profileId ? `/profiles/${profileId}` : '/events')
    return
  }

  if (!accessToken || !refreshToken) {
    error.value = t('authCallback.invalidParams')
    return
  }

  try {
    await authStore.initFromOAuth(accessToken, refreshToken)
    router.replace(sanitizeRedirectPath(redirectQuery || '/events'))
  } catch {
    error.value = t('authCallback.failed')
  }
})

async function submitEmail(): Promise<void> {
  if (!needsEmail.value || submitting.value) {
    return
  }

  const trimmedEmail = email.value.trim()
  if (!trimmedEmail || !trimmedEmail.includes('@')) {
    error.value = t('authCallback.invalidEmail')
    return
  }

  submitting.value = true
  error.value = ''
  try {
    await authStore.completeBnetSignup(pendingToken.value, trimmedEmail)
    router.replace({ name: 'verify-email-pending', query: { email: trimmedEmail } })
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('authCallback.completeFailed')
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <main class="app-shell auth-shell">
    <section class="card auth-card">
      <template v-if="needsEmail">
        <h1>{{ t('authCallback.oneMoreStep') }}</h1>
        <p class="muted">
          {{ t('authCallback.bnetNoEmail') }}
          <strong v-if="battletag">{{ battletag }}</strong>
          <span v-else>{{ t('authCallback.bnetNoEmailAnon') }}</span>.
          {{ t('authCallback.bnetEmailInstruction') }}
        </p>
        <form class="email-form" @submit.prevent="submitEmail">
          <label class="field-label" for="oauth-email">{{ t('authCallback.emailLabel') }}</label>
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
            {{ submitting ? t('authCallback.completing') : t('authCallback.completeSignup') }}
          </button>
        </form>
        <p v-if="error" class="status status-error">{{ error }}</p>
      </template>
      <template v-else-if="error">
        <div class="oauth-error">
          <span class="material-symbols-rounded oauth-error-icon" aria-hidden="true">error</span>
          <h2 class="oauth-error-title">{{ t('authCallback.errorTitle') }}</h2>
          <p class="oauth-error-message">{{ error }}</p>
          <RouterLink :to="returnPath" class="oauth-error-back">{{ t('authCallback.goBack') }}</RouterLink>
        </div>
      </template>
      <template v-else>
        <p class="muted">{{ t('authCallback.signingIn') }}</p>
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

.oauth-error {
  display: grid;
  justify-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0;
}

.oauth-error-icon {
  font-size: 2.8rem;
  color: var(--status-error, #e05252);
  font-variation-settings: 'FILL' 1, 'wght' 300, 'GRAD' 0, 'opsz' 48;
}

.oauth-error-title {
  margin: 0;
  font-size: 1.2rem;
}

.oauth-error-message {
  margin: 0;
  color: var(--ink-muted);
  font-size: 0.95rem;
}

.oauth-error-back {
  font-size: 0.9rem;
  color: var(--ink-muted);
  text-decoration: none;
}

.oauth-error-back:hover {
  color: var(--ink-1);
  text-decoration: underline;
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
