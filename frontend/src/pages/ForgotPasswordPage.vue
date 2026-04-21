<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ApiHttpError } from '../lib/api'
import { useAuthStore } from '../stores/auth'

const { t } = useI18n()
const authStore = useAuthStore()

const email = ref('')
const submitting = ref(false)
const submitted = ref(false)
const submitError = ref('')

async function submit() {
  if (!email.value.trim() || submitting.value) return
  submitting.value = true
  submitError.value = ''
  try {
    await authStore.forgotPassword(email.value.trim())
    submitted.value = true
  } catch (err) {
    if (err instanceof ApiHttpError && err.status === 429) {
      submitError.value = err.message
    } else {
      submitError.value = t('forgotPassword.submitError')
    }
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <main class="app-shell fp-shell">
    <section class="card fp-card">
      <template v-if="!submitted">
        <header class="fp-header">
          <span class="material-symbols-rounded fp-icon" aria-hidden="true">lock_reset</span>
          <h1 class="fp-title">{{ t('forgotPassword.title') }}</h1>
          <p class="muted">{{ t('forgotPassword.subtitle') }}</p>
        </header>

        <form class="grid-form" @submit.prevent="submit">
          <label>
            {{ t('forgotPassword.emailLabel') }}
            <input
              v-model="email"
              type="email"
              :placeholder="t('forgotPassword.emailPlaceholder')"
              required
              :disabled="submitting"
            />
          </label>
          <button type="submit" class="btn-primary" :disabled="submitting || !email.trim()">
            {{ submitting ? t('forgotPassword.submitting') : t('forgotPassword.submitBtn') }}
          </button>
          <p v-if="submitError" class="field-error" role="alert">{{ submitError }}</p>
        </form>
      </template>

      <template v-else>
        <span class="material-symbols-rounded fp-icon fp-icon-ok" aria-hidden="true">mark_email_read</span>
        <h1 class="fp-title">{{ t('forgotPassword.successTitle') }}</h1>
        <p class="muted">{{ t('forgotPassword.successSubtitle') }}</p>
        <p class="muted fp-note">{{ t('forgotPassword.rateLimitHint') }}</p>
        <p class="muted fp-note">{{ t('forgotPassword.spamHint') }}</p>
      </template>

      <p class="fp-back">
        <RouterLink :to="{ name: 'login' }">← {{ t('forgotPassword.backToLogin') }}</RouterLink>
      </p>
    </section>
  </main>
</template>

<style scoped>
.fp-shell {
  min-height: calc(100vh - 220px);
  display: grid;
  align-content: center;
}

.fp-card {
  max-width: 440px;
  width: 100%;
  margin: 0 auto;
  display: grid;
  gap: 1.2rem;
}

.fp-header {
  display: grid;
  gap: 0.4rem;
  text-align: center;
}

.fp-icon {
  font-size: 2.8rem;
  color: var(--brand-1);
  justify-self: center;
}

.fp-icon-ok {
  justify-self: center;
}

.fp-title {
  margin: 0;
}

.fp-note {
  margin: 0;
  font-size: 0.9rem;
}

.fp-back {
  margin: 0;
  text-align: center;
  font-size: 0.85rem;
}

.fp-back a {
  color: var(--ink-muted);
  text-decoration: none;
}

.fp-back a:hover {
  color: var(--brand-1);
  text-decoration: underline;
}
</style>
