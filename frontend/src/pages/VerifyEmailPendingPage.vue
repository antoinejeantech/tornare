<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../stores/auth'

const { t } = useI18n()
const route = useRoute()
const authStore = useAuthStore()

const email = ref('')
const submitting = ref(false)
const message = ref('')
const error = ref('')
const canResend = computed(() => email.value.trim().length > 0)

onMounted(() => {
  email.value = typeof route.query.email === 'string' ? route.query.email : ''
})

async function resend() {
  if (!canResend.value || submitting.value) return

  submitting.value = true
  message.value = ''
  error.value = ''
  try {
    const result = await authStore.resendVerification(email.value.trim())
    message.value = result.message
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('verifyEmailPending.resendFailed')
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <main class="app-shell pending-shell">
    <section class="card pending-card">
      <span class="material-symbols-rounded pending-icon" aria-hidden="true">mark_email_unread</span>
      <h1 class="pending-title">{{ t('verifyEmailPending.title') }}</h1>
      <p class="muted">{{ t('verifyEmailPending.subtitle') }}</p>
      <p v-if="email" class="pending-email">{{ email }}</p>
      <p v-else class="status status-error">{{ t('verifyEmailPending.missingEmail') }}</p>

      <p class="muted pending-note">{{ t('verifyEmailPending.note') }}</p>

      <div class="pending-resend">
        <button class="btn-secondary" type="button" :disabled="submitting || !canResend" @click="resend">
          {{ submitting ? t('verifyEmailPending.sending') : t('verifyEmailPending.resendBtn') }}
        </button>
        <p v-if="!canResend" class="muted pending-resend-help">{{ t('verifyEmailPending.resendUnavailable') }}</p>
      </div>

      <p v-if="message" class="status status-success">{{ message }}</p>
      <p v-if="message" class="muted pending-note">{{ t('verifyEmailPending.rateLimitHint') }}</p>
      <p v-if="message" class="muted pending-note">{{ t('verifyEmailPending.spamHint') }}</p>
      <p v-if="error" class="status status-error">{{ error }}</p>
    </section>
  </main>
</template>

<style scoped>
.pending-shell {
  min-height: calc(100vh - 220px);
  display: grid;
  align-content: center;
}

.pending-card {
  max-width: 480px;
  width: 100%;
  margin: 0 auto;
  display: grid;
  gap: 0.9rem;
  text-align: center;
}

.pending-icon {
  font-size: 3rem;
  color: var(--brand-1);
  justify-self: center;
}

.pending-title {
  margin: 0;
}

.pending-email {
  font-weight: 600;
  margin: 0;
}

.pending-note {
  font-size: 0.85rem;
  margin: 0;
}

.pending-resend {
  border-top: 1px solid var(--line);
  padding-top: 1rem;
}

.pending-resend-help {
  margin: 0.6rem 0 0;
  font-size: 0.85rem;
}
</style>
