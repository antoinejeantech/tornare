<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../stores/auth'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const token = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const submitting = ref(false)
const success = ref(false)
const error = ref('')

onMounted(() => {
  token.value = typeof route.query.token === 'string' ? route.query.token : ''
})

const canSubmit = () =>
  newPassword.value.length >= 8 && newPassword.value === confirmPassword.value

async function submit() {
  if (!canSubmit() || submitting.value) return
  submitting.value = true
  error.value = ''
  try {
    await authStore.resetPassword(token.value, newPassword.value, confirmPassword.value)
    success.value = true
    setTimeout(() => router.push({ name: 'login' }), 2000)
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('resetPassword.invalidToken')
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <main class="app-shell rp-shell">
    <section class="card rp-card">

      <!-- No token in URL -->
      <template v-if="!token">
        <span class="material-symbols-rounded rp-icon rp-icon-err" aria-hidden="true">link_off</span>
        <h1 class="rp-title">{{ t('resetPassword.errorTitle') }}</h1>
        <p class="muted">{{ t('resetPassword.missingToken') }}</p>
        <RouterLink :to="{ name: 'forgot-password' }" class="btn-secondary rp-cta">
          {{ t('resetPassword.requestNew') }}
        </RouterLink>
      </template>

      <!-- Success state -->
      <template v-else-if="success">
        <span class="material-symbols-rounded rp-icon rp-icon-ok" aria-hidden="true">check_circle</span>
        <h1 class="rp-title">{{ t('resetPassword.successTitle') }}</h1>
        <p class="muted">{{ t('resetPassword.successSubtitle') }}</p>
      </template>

      <!-- Form -->
      <template v-else>
        <header class="rp-header">
          <span class="material-symbols-rounded rp-icon" aria-hidden="true">lock_reset</span>
          <h1 class="rp-title">{{ t('resetPassword.title') }}</h1>
        </header>

        <p v-if="error" class="status status-error">{{ error }}</p>
        <p v-if="error" class="rp-request-new">
          <RouterLink :to="{ name: 'forgot-password' }">{{ t('resetPassword.requestNew') }}</RouterLink>
        </p>

        <form class="grid-form" @submit.prevent="submit">
          <label>
            {{ t('resetPassword.newPasswordLabel') }}
            <input v-model="newPassword" type="password" minlength="8" required :disabled="submitting" />
          </label>
          <label>
            {{ t('resetPassword.confirmLabel') }}
            <input v-model="confirmPassword" type="password" minlength="8" required :disabled="submitting" />
          </label>
          <button type="submit" class="btn-primary" :disabled="!canSubmit() || submitting">
            {{ submitting ? t('resetPassword.submitting') : t('resetPassword.submitBtn') }}
          </button>
        </form>
      </template>

    </section>
  </main>
</template>

<style scoped>
.rp-shell {
  min-height: calc(100vh - 220px);
  display: grid;
  align-content: center;
}

.rp-card {
  max-width: 440px;
  width: 100%;
  margin: 0 auto;
  display: grid;
  gap: 1.2rem;
  text-align: center;
}

.rp-header {
  display: grid;
  gap: 0.4rem;
}

.rp-icon {
  font-size: 2.8rem;
  color: var(--brand-1);
  justify-self: center;
}

.rp-icon-ok {
  color: var(--ok-ink);
}

.rp-icon-err {
  color: var(--err-ink);
}

.rp-title {
  margin: 0;
}

.rp-cta {
  text-decoration: none;
}

.rp-request-new {
  margin: -0.5rem 0 0;
  font-size: 0.85rem;
  text-align: center;
}

.rp-request-new a {
  color: var(--ink-muted);
  text-decoration: underline;
}

.grid-form {
  text-align: left;
}
</style>
