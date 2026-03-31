<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { getApiBase } from '../lib/api'
import battlenetLogo from '../assets/branding/bnet-logo.png'

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
    return mode.value === 'register' ? 'Creating account...' : 'Signing in...'
  }

  return mode.value === 'register' ? 'Create account' : 'Sign in'
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

    const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/events'
    router.push(redirect)
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Authentication failed'
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
        <h1 class="page-title">{{ mode === 'register' ? 'Create Account' : 'Sign In' }}</h1>
        <p class="muted">Use your email and password to access your events.</p>
      </header>

      <p v-if="error" class="status status-error">{{ error }}</p>

      <div class="auth-bnet">
        <button type="button" class="btn-bnet" @click="loginWithBnet">
          <img class="btn-bnet-logo" :src="battlenetLogo" alt="" aria-hidden="true" />
          <span class="btn-bnet-label">Sign in with Battle.net</span>
        </button>
        <button type="button" class="btn-discord" @click="loginWithDiscord">
          <svg class="btn-discord-logo" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/></svg>
          <span class="btn-discord-label">Sign in with Discord</span>
        </button>
      </div>

      <div class="auth-divider" aria-hidden="true">
        <span>{{ mode === 'register' ? 'or create account with email' : 'or sign in with email' }}</span>
      </div>

      <form class="grid-form" @submit.prevent="submit">
        <label v-if="mode === 'register'">
          Username
          <input v-model="username" placeholder="antoine" />
        </label>
        <label v-if="mode === 'register'">
          Display name
          <input v-model="displayName" placeholder="Antoine" />
        </label>
        <label>
          Email
          <input v-model="email" type="email" placeholder="you@example.com" />
        </label>
        <label>
          Password
          <input v-model="password" type="password" placeholder="At least 8 characters" />
        </label>
        <label v-if="mode === 'register'">
          Confirm password
          <input v-model="passwordConfirm" type="password" placeholder="Repeat your password" />
        </label>
        <button type="submit" class="btn-primary" :disabled="!canSubmit || submitting">{{ submitLabel }}</button>
      </form>

      <p class="auth-switch-hint">
        <template v-if="mode === 'login'">
          No account yet?
          <RouterLink :to="{ name: 'register', query: route.query }">Create one</RouterLink>
        </template>
        <template v-else>
          Already have an account?
          <RouterLink :to="{ name: 'login', query: route.query }">Sign in</RouterLink>
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

.btn-bnet {
  width: 100%;
  border: none;
  border-radius: var(--radius-md);
  padding: 0.72rem 1.1rem;
  background: #148eff;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.65rem;
  cursor: pointer;
  transition: background 120ms ease, box-shadow 120ms ease;
  box-shadow: 0 2px 8px rgb(20 142 255 / 32%);
}

.btn-bnet:hover {
  background: #1a9aff;
  box-shadow: 0 3px 12px rgb(20 142 255 / 44%);
}

.btn-bnet:active {
  background: #0e7de0;
  box-shadow: 0 1px 4px rgb(20 142 255 / 24%);
}

.btn-bnet-logo {
  width: 1.55rem;
  height: 1.55rem;
  display: block;
  border-radius: var(--radius-pill);
  flex-shrink: 0;
  background: #fff;
  padding: 0.18rem;
}

.btn-bnet-label {
  font-weight: 700;
  font-size: 0.97rem;
  letter-spacing: 0.01em;
}

.btn-discord {
  width: 100%;
  border: none;
  border-radius: var(--radius-md);
  padding: 0.72rem 1.1rem;
  background: #5865f2;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.65rem;
  cursor: pointer;
  transition: background 120ms ease, box-shadow 120ms ease;
  box-shadow: 0 2px 8px rgb(88 101 242 / 32%);
}

.btn-discord:hover {
  background: #6470f3;
  box-shadow: 0 3px 12px rgb(88 101 242 / 44%);
}

.btn-discord:active {
  background: #4752c4;
  box-shadow: 0 1px 4px rgb(88 101 242 / 24%);
}

.btn-discord-logo {
  width: 1.45rem;
  height: 1.45rem;
  flex-shrink: 0;
}

.btn-discord-label {
  font-weight: 700;
  font-size: 0.97rem;
  letter-spacing: 0.01em;
}

@media (prefers-color-scheme: dark) {
  /* BNet and Discord brand colors work on both light and dark — no override needed */
}
</style>
