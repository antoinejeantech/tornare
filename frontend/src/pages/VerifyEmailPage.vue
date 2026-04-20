<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../stores/auth'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const state = ref<'verifying' | 'success' | 'error'>('verifying')
const errorMessage = ref('')

onMounted(async () => {
  const token = typeof route.query.token === 'string' ? route.query.token.trim() : ''
  if (!token) {
    state.value = 'error'
    errorMessage.value = t('verifyEmail.missingToken')
    return
  }

  try {
    await authStore.verifyEmail(token)
    state.value = 'success'
    setTimeout(() => {
      router.replace({ name: 'onboarding' })
    }, 2000)
  } catch (err) {
    state.value = 'error'
    errorMessage.value = err instanceof Error ? err.message : t('verifyEmail.failed')
  }
})
</script>

<template>
  <main class="app-shell verify-shell">
    <section class="card verify-card">
      <template v-if="state === 'verifying'">
        <span class="material-symbols-rounded verify-icon spinning" aria-hidden="true">progress_activity</span>
        <p class="muted">{{ t('verifyEmail.verifying') }}</p>
      </template>

      <template v-else-if="state === 'success'">
        <span class="material-symbols-rounded verify-icon verify-icon--success" aria-hidden="true">check_circle</span>
        <h1 class="verify-title">{{ t('verifyEmail.successTitle') }}</h1>
        <p class="muted">{{ t('verifyEmail.successSubtitle') }}</p>
      </template>

      <template v-else>
        <span class="material-symbols-rounded verify-icon verify-icon--error" aria-hidden="true">error</span>
        <h1 class="verify-title">{{ t('verifyEmail.errorTitle') }}</h1>
        <p class="status status-error">{{ errorMessage }}</p>
        <RouterLink :to="{ name: 'verify-email-pending' }" class="btn-secondary">
          {{ t('verifyEmail.requestNew') }}
        </RouterLink>
      </template>
    </section>
  </main>
</template>

<style scoped>
.verify-shell {
  min-height: calc(100vh - 220px);
  display: grid;
  align-content: center;
}

.verify-card {
  max-width: 420px;
  width: 100%;
  margin: 0 auto;
  display: grid;
  gap: 0.9rem;
  text-align: center;
}

.verify-icon {
  font-size: 3.5rem;
  justify-self: center;
  color: var(--ink-muted);
}

.verify-icon--success {
  color: var(--status-success, #22c55e);
}

.verify-icon--error {
  color: var(--status-error, #ef4444);
}

.verify-title {
  margin: 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.spinning {
  animation: spin 1s linear infinite;
}
</style>
