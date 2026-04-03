<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../stores/auth'
import { getApiBase } from '../lib/api'
import DiscordAuthButton from '../components/ui/DiscordAuthButton.vue'
import BnetAuthButton from '../components/ui/BnetAuthButton.vue'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const mode = computed(() => route.name === 'register' ? 'register' : 'login')
watch(mode, () => { error.value = '' })
const email = ref('')
const password = ref('')
const passwordConfirm = ref('')
const username = ref('')
const displayName = ref('')
const error = ref('')
const submitting = ref(false)

const canSubmit = computed(() => {
  const emailOk = email.value.trim().length > 0
  const passwordOk = password.value.length >= 8

  if (mode.value === 'register') {
    const usernameValue = username.value.trim().toLowerCase()
    const usernameOk =
      usernameValue.length >= 3 &&
      usernameValue.length <= 24 &&
      /^[a-z0-9_]+$/.test(usernameValue)

    return (
      emailOk &&
      passwordOk &&
      password.value === passwordConfirm.value &&
      usernameOk &&
      displayName.value.trim().length > 0
    )
  }

  return emailOk && passwordOk
})

const submitLabel = computed(() => {
  if (submitting.value) {
    return mode.value === 'register' ? t('auth.creating') : t('auth.signingIn')
  }
  return mode.value === 'register' ? t('auth.createBtn') : t('auth.signInBtn')
})

async function submit() {
  if (!canSubmit.value || submitting.value) {
    return
  }

  submitting.value = true
  error.value = ''

  try {
    if (mode.value === 'register') {
      await authStore.register({
        email: email.value.trim(),
        password: password.value,
        password_confirm: passwordConfirm.value,
        username: username.value.trim().toLowerCase(),
        display_name: displayName.value.trim(),
      })
    } else {
      await authStore.login({
        email: email.value.trim(),
        password: password.value,
      })
    }

    if (mode.value === 'register') {
      router.push({ name: 'onboarding' })
    } else {
      const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/events'
      router.push(redirect)
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('auth.authFailed')
  } finally {
    submitting.value = false
  }
}

function loginWithBnet() {
  window.location.href = `${getApiBase()}/api/auth/battlenet/authorize`
}

function loginWithDiscord() {
  window.location.href = `${getApiBase()}/api/auth/discord/authorize`
}
</script>

<template>
  <main class="app-shell auth-shell">
    <section class="card auth-card">
      <header class="auth-header">
        <h1 class="page-title">{{ mode === 'register' ? t('auth.createAccount') : t('auth.signIn') }}</h1>
        <p class="muted">{{ t('auth.subtitle') }}</p>
      </header>

      <p v-if="error" class="status status-error">{{ error }}</p>

      <div class="auth-bnet">
        <DiscordAuthButton :label="t('auth.signInDiscord')" :recommended="true" @click="loginWithDiscord" />
        <BnetAuthButton :label="t('auth.signInBnet')" @click="loginWithBnet" />
      </div>

      <div class="auth-divider" aria-hidden="true">
        <span>{{ mode === 'register' ? t('auth.dividerRegister') : t('auth.dividerLogin') }}</span>
      </div>

      <form class="grid-form" @submit.prevent="submit">
        <label v-if="mode === 'register'">
          {{ t('auth.username') }}
          <input v-model="username" :placeholder="t('auth.usernamePlaceholder')" />
        </label>
        <label v-if="mode === 'register'">
          {{ t('auth.displayName') }}
          <input v-model="displayName" :placeholder="t('auth.displayNamePlaceholder')" />
        </label>
        <label>
          {{ t('auth.email') }}
          <input v-model="email" type="email" :placeholder="t('auth.emailPlaceholder')" />
        </label>
        <label>
          {{ t('auth.password') }}
          <input v-model="password" type="password" :placeholder="t('auth.passwordPlaceholder')" />
        </label>
        <label v-if="mode === 'register'">
          {{ t('auth.confirmPassword') }}
          <input v-model="passwordConfirm" type="password" :placeholder="t('auth.confirmPasswordPlaceholder')" />
        </label>
        <button type="submit" class="btn-primary" :disabled="!canSubmit || submitting">{{ submitLabel }}</button>
      </form>

      <p class="auth-switch-hint">
        <template v-if="mode === 'login'">
          {{ t('auth.noAccount') }}
          <RouterLink :to="{ name: 'register', query: route.query }">{{ t('auth.createOne') }}</RouterLink>
        </template>
        <template v-else>
          {{ t('auth.hasAccount') }}
          <RouterLink :to="{ name: 'login', query: route.query }">{{ t('auth.signInBtn') }}</RouterLink>
        </template>
      </p>
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
  width: 100%;
  margin: 0 auto;
  display: grid;
  gap: 0.8rem;
}

.auth-header h1 {
  margin-bottom: 0.35rem;
}

.auth-header p {
  margin: 0;
}

.auth-switch-hint {
  margin: 0;
  text-align: center;
  font-size: 0.88rem;
  color: var(--ink-muted);
}

.auth-switch-hint a {
  color: var(--brand-1);
  font-weight: 600;
  text-decoration: none;
}

.auth-switch-hint a:hover {
  text-decoration: underline;
}

.auth-bnet {
  display: grid;
  gap: 0.5rem;
}

.auth-divider {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 0.7rem;
  color: var(--ink-muted);
  font-size: 0.82rem;
}

.auth-divider::before,
.auth-divider::after {
  content: '';
  height: 1px;
  background: color-mix(in srgb, var(--line) 55%, transparent 45%);
}

@media (prefers-color-scheme: dark) {
  /* BNet and Discord brand colors work on both light and dark — no override needed */
}
</style>
