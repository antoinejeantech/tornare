<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import battlenetLogo from '../assets/branding/bnet-logo.png'
import AppBadge from '../components/ui/AppBadge.vue'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const mode = ref('login')
const email = ref('')
const password = ref('')
const passwordConfirm = ref('')
const username = ref('')
const displayName = ref('')
const error = ref('')
const submitting = ref(false)
const publicSignupFlag = String(import.meta.env.VITE_PUBLIC_SIGNUP_ENABLED || '').trim().toLowerCase()
const publicSignupEnabled = publicSignupFlag
  ? publicSignupFlag === 'true'
  : Boolean(import.meta.env.DEV)

const canSubmit = computed(() => {
  const emailOk = email.value.trim().length > 0
  const passwordOk = password.value.length >= 8

  if (mode.value === 'register') {
    if (!publicSignupEnabled) {
      return false
    }

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
      if (!publicSignupEnabled) {
        error.value = 'Public signup will be available soon.'
        return
      }

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

function switchMode(nextMode: string) {
  if (nextMode === 'register' && !publicSignupEnabled) {
    error.value = 'Public signup will be available soon.'
    return
  }

  mode.value = nextMode
  error.value = ''
  if (nextMode !== 'register') {
    passwordConfirm.value = ''
  }
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

      <div v-if="!publicSignupEnabled" class="auth-signup-lock" role="status" aria-live="polite">
        <span class="material-symbols-rounded" aria-hidden="true">lock</span>
        <div>
          <strong>Public signup is currently disabled</strong>
          <p class="muted">It will be available to everyone soon. Login is active if you already have an account.</p>
        </div>
      </div>

      <div class="auth-soon">
        <button type="button" class="btn-bnet" disabled aria-disabled="true" title="Coming soon">
          <span class="btn-bnet-brand">
            <img class="btn-bnet-logo" :src="battlenetLogo" alt="Battle.net" />
            <span class="btn-bnet-label">Connect with Battle.net</span>
          </span>
          <AppBadge label="Coming soon" radius="pill" bg="linear-gradient(135deg, #ef5f00, #f28b2f)" color="#fff" />
        </button>
        <p class="muted auth-soon-note">Social login is on the roadmap and will arrive in a future update.</p>
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

      <div class="auth-switch-row">
        <button
          class="btn-secondary"
          :disabled="mode === 'login'"
          @click="switchMode('login')"
          type="button"
        >
          Login
        </button>
        <button
          class="btn-secondary"
          :class="{ 'btn-disabled-feature': !publicSignupEnabled }"
          :disabled="mode === 'register' || !publicSignupEnabled"
          :title="publicSignupEnabled ? 'Register' : 'Public signup will be available soon'"
          @click="switchMode('register')"
          type="button"
        >
          {{ publicSignupEnabled ? 'Register' : 'Register (Soon)' }}
        </button>
      </div>
      <p v-if="!publicSignupEnabled" class="muted auth-signup-disabled-note">
        Public signup will be available soon.
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

.auth-switch-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.55rem;
}

.auth-soon {
  display: grid;
  gap: 0.4rem;
}

.btn-bnet {
  border: 1px dashed color-mix(in srgb, var(--line) 68%, #f06414 32%);
  border-radius: var(--radius-md);
  padding: 0.7rem 0.9rem;
  background: linear-gradient(120deg, #fff5ed, #ffe9d7);
  color: #5c2400;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  cursor: not-allowed;
  opacity: 1;
}

.btn-bnet-brand {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
}

.btn-bnet-logo {
  width: 1.55rem;
  height: 1.55rem;
  display: block;
  background: #fff;
  border: 1px solid color-mix(in srgb, #0b5ed7 22%, #ffffff 78%);
  border-radius: var(--radius-pill);
  padding: 0.2rem;
  box-shadow: 0 1px 2px rgb(0 0 0 / 12%);
}

.btn-bnet-label {
  font-weight: 780;
  letter-spacing: 0.01em;
}

.auth-soon-note {
  margin: 0;
  font-size: 0.88rem;
}

.auth-signup-disabled-note {
  margin: -0.1rem 0 0;
  font-size: 0.86rem;
}

.auth-signup-lock {
  border: 1px dashed color-mix(in srgb, var(--line) 64%, #f08b2f 36%);
  background: linear-gradient(130deg, color-mix(in srgb, var(--card) 78%, #fff1df 22%), color-mix(in srgb, var(--card) 86%, #ffe5cc 14%));
  border-radius: var(--radius-md);
  padding: 0.6rem 0.72rem;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 0.55rem;
  align-items: start;
}

.auth-signup-lock strong {
  display: block;
  font-size: 0.9rem;
}

.auth-signup-lock p {
  margin: 0.12rem 0 0;
  font-size: 0.84rem;
}

.btn-disabled-feature {
  border-style: dashed;
  opacity: 0.75;
}

@media (prefers-color-scheme: dark) {
  .btn-bnet {
    border-color: color-mix(in srgb, var(--line) 72%, #2e9bff 28%);
    background: linear-gradient(120deg, #1b2433, #182134);
    color: #dbe9ff;
  }

  .btn-bnet-logo {
    background: #0f1725;
    border-color: color-mix(in srgb, #2e9bff 45%, #0f1725 55%);
  }

}
</style>
