<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { getApiBase } from '../lib/api'
import BnetIcon from '../components/ui/BnetIcon.vue'
import DiscordIcon from '../components/ui/DiscordIcon.vue'

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
          <BnetIcon class="btn-bnet-logo" />
          <span class="btn-bnet-label">Sign in with Battle.net</span>
        </button>
        <button type="button" class="btn-discord" @click="loginWithDiscord">
          <DiscordIcon class="btn-discord-logo" />
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
  flex-shrink: 0;
  color: white;
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
