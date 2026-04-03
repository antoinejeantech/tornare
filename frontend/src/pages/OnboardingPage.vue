<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../stores/auth'
import { getRankIcon, overwatchRanks } from '../lib/ranks'
import { apiCall } from '../lib/api'
import type { AuthUser, OverwatchRank } from '../types'
import DiscordAuthButton from '../components/ui/DiscordAuthButton.vue'
import BnetAuthButton from '../components/ui/BnetAuthButton.vue'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const TOTAL_STEPS = 3

const step = computed(() => {
  const raw = Number(route.query.step)
  if (raw >= 1 && raw <= TOTAL_STEPS) return raw
  return 1
})

// ── Step 3 state ────────────────────────────────────────────────────────
const rankTank = ref<OverwatchRank>(authStore.user?.rank_tank ?? 'Unranked')
const rankDps = ref<OverwatchRank>(authStore.user?.rank_dps ?? 'Unranked')
const rankSupport = ref<OverwatchRank>(authStore.user?.rank_support ?? 'Unranked')
const savingRanks = ref(false)
const connectingDiscord = ref(false)
const connectingBnet = ref(false)

// ── Helpers ────────────────────────────────────────────────────────────
function goToStep(n: number) {
  if (n > TOTAL_STEPS) {
    router.push({ name: 'events' })
  } else {
    router.push({ name: 'onboarding', query: { step: n } })
  }
}

function skip() {
  goToStep(step.value + 1)
}

// ── Step 1: Discord ────────────────────────────────────────────────────
async function connectDiscord() {
  connectingDiscord.value = true
  sessionStorage.setItem('onboarding_resume_step', '2')
  try {
    await authStore.connectDiscordInit()
    // navigates away — no finally needed
  } catch {
    connectingDiscord.value = false
    sessionStorage.removeItem('onboarding_resume_step')
  }
}

// ── Step 2: Battle.net ─────────────────────────────────────────────────
async function connectBnet() {
  connectingBnet.value = true
  sessionStorage.setItem('onboarding_resume_step', '3')
  try {
    await authStore.connectBnetInit()
    // navigates away — no finally needed
  } catch {
    connectingBnet.value = false
    sessionStorage.removeItem('onboarding_resume_step')
  }
}

// ── Step 3: Ranks ──────────────────────────────────────────────────────
async function saveRanks() {
  if (savingRanks.value || !authStore.user) return
  savingRanks.value = true
  try {
    const u = authStore.user
    const updated = await apiCall<AuthUser>(`/api/users/${u.id}`, {
      method: 'PUT',
      body: JSON.stringify({
        username: u.username,
        display_name: u.display_name,
        email: u.email,
        battletag: u.battletag ?? null,
        rank_tank: rankTank.value,
        rank_dps: rankDps.value,
        rank_support: rankSupport.value,
        new_password: null,
        new_password_confirm: null,
      }),
    })
    authStore.user = updated
    router.push({ name: 'events' })
  } finally {
    savingRanks.value = false
  }
}
</script>

<template>
  <main class="app-shell onboarding-shell">
    <section class="card onboarding-card">

      <!-- Step indicator -->
      <div class="onboarding-steps-bar" aria-label="Progress">
        <div
          v-for="n in TOTAL_STEPS"
          :key="n"
          class="onboarding-step-dot"
          :class="{ active: n === step, done: n < step }"
        />
      </div>

      <p class="onboarding-step-label muted">
        {{ t('onboarding.stepOf', { step, total: TOTAL_STEPS }) }}
      </p>

      <!-- ── Step 1: Discord ────────────────────────────────────────── -->
      <template v-if="step === 1">
        <header class="onboarding-header">
          <h1 class="page-title">{{ t('onboarding.step1Title') }}</h1>
          <p class="muted">{{ t('onboarding.step1Desc') }}</p>
        </header>

        <div v-if="authStore.user?.has_discord_identity" class="onboarding-connected-state">
          <span class="material-symbols-rounded onboarding-check-icon" aria-hidden="true">check_circle</span>
          <div>
            <strong>{{ t('onboarding.step1Done') }}</strong>
            <p v-if="authStore.user?.discord_username" class="muted onboarding-connected-name">
              {{ t('onboarding.step1Connected') }} {{ authStore.user.discord_username }}
            </p>
          </div>
        </div>

        <DiscordAuthButton
          v-else
          :label="connectingDiscord ? t('onboarding.connectingDiscord') : t('onboarding.step1Cta')"
          :disabled="connectingDiscord"
          @click="connectDiscord"
        />

        <div class="onboarding-actions">
          <button type="button" class="btn-ghost" @click="skip">{{ t('onboarding.skip') }}</button>
          <button
            v-if="authStore.user?.has_discord_identity"
            type="button"
            class="btn-primary"
            @click="goToStep(2)"
          >
            {{ t('onboarding.continue') }}
          </button>
        </div>
      </template>

      <!-- ── Step 2: Battle.net ─────────────────────────────────────── -->
      <template v-else-if="step === 2">
        <header class="onboarding-header">
          <h1 class="page-title">{{ t('onboarding.step2Title') }}</h1>
          <p class="muted">{{ t('onboarding.step2Desc') }}</p>
        </header>

        <div v-if="authStore.user?.battletag" class="onboarding-connected-state">
          <span class="material-symbols-rounded onboarding-check-icon" aria-hidden="true">check_circle</span>
          <div>
            <strong>{{ t('onboarding.step2Done') }}</strong>
            <p class="muted onboarding-connected-name">
              {{ t('onboarding.step2Connected') }} {{ authStore.user.battletag }}
            </p>
          </div>
        </div>

        <BnetAuthButton
          v-else
          :label="connectingBnet ? t('onboarding.connectingBnet') : t('onboarding.step2Cta')"
          :disabled="connectingBnet"
          @click="connectBnet"
        />

        <div class="onboarding-actions">
          <button type="button" class="btn-ghost" @click="skip">{{ t('onboarding.skip') }}</button>
          <button
            v-if="authStore.user?.battletag"
            type="button"
            class="btn-primary"
            @click="goToStep(3)"
          >
            {{ t('onboarding.continue') }}
          </button>
        </div>
      </template>

      <!-- ── Step 3: Roles & Ranks ──────────────────────────────────── -->
      <template v-else-if="step === 3">
        <header class="onboarding-header">
          <h1 class="page-title">{{ t('onboarding.step3Title') }}</h1>
          <p class="muted">{{ t('onboarding.step3Desc') }}</p>
        </header>

        <div class="onboarding-ranks">
          <label class="onboarding-rank-row">
            <span class="onboarding-rank-label">
              <span class="material-symbols-rounded" aria-hidden="true">shield</span>
              {{ t('onboarding.step3Tank') }}
            </span>
            <div class="onboarding-rank-select-wrap">
              <img :src="getRankIcon(rankTank)" class="onboarding-rank-icon" alt="" aria-hidden="true" />
              <select v-model="rankTank" class="onboarding-rank-select">
                <option v-for="rank in overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
              </select>
            </div>
          </label>

          <label class="onboarding-rank-row">
            <span class="onboarding-rank-label">
              <span class="material-symbols-rounded" aria-hidden="true">swords</span>
              {{ t('onboarding.step3Dps') }}
            </span>
            <div class="onboarding-rank-select-wrap">
              <img :src="getRankIcon(rankDps)" class="onboarding-rank-icon" alt="" aria-hidden="true" />
              <select v-model="rankDps" class="onboarding-rank-select">
                <option v-for="rank in overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
              </select>
            </div>
          </label>

          <label class="onboarding-rank-row">
            <span class="onboarding-rank-label">
              <span class="material-symbols-rounded" aria-hidden="true">health_cross</span>
              {{ t('onboarding.step3Support') }}
            </span>
            <div class="onboarding-rank-select-wrap">
              <img :src="getRankIcon(rankSupport)" class="onboarding-rank-icon" alt="" aria-hidden="true" />
              <select v-model="rankSupport" class="onboarding-rank-select">
                <option v-for="rank in overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
              </select>
            </div>
          </label>
        </div>

        <div class="onboarding-actions">
          <button type="button" class="btn-ghost" @click="router.push({ name: 'events' })">
            {{ t('onboarding.skip') }}
          </button>
          <button type="button" class="btn-primary" :disabled="savingRanks" @click="saveRanks">
            {{ savingRanks ? t('onboarding.step3Saving') : t('onboarding.step3Save') }}
          </button>
        </div>
      </template>

    </section>
  </main>
</template>

<style scoped>
.onboarding-shell {
  min-height: calc(100vh - 220px);
  display: grid;
  align-content: center;
}

.onboarding-card {
  max-width: 520px;
  width: 100%;
  margin: 0 auto;
  display: grid;
  gap: 1.2rem;
}

/* ── Step indicator ── */
.onboarding-steps-bar {
  display: flex;
  gap: 0.45rem;
  justify-content: center;
}

.onboarding-step-dot {
  width: 0.55rem;
  height: 0.55rem;
  border-radius: 50%;
  background: var(--line);
  transition: background 200ms ease;
}

.onboarding-step-dot.active {
  background: var(--brand-1);
  width: 1.4rem;
  border-radius: 4px;
}

.onboarding-step-dot.done {
  background: color-mix(in srgb, var(--brand-1) 50%, var(--line) 50%);
}

.onboarding-step-label {
  text-align: center;
  font-size: 0.8rem;
  margin: 0;
}

/* ── Header ── */
.onboarding-header {
  text-align: center;
}

.onboarding-header h1 {
  margin-bottom: 0.35rem;
}

.onboarding-header p {
  margin: 0;
}

/* ── Connected state ── */
.onboarding-connected-state {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.85rem 1rem;
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--status-success, #3fa96a) 10%, transparent 90%);
  border: 1px solid color-mix(in srgb, var(--status-success, #3fa96a) 30%, transparent 70%);
}

.onboarding-check-icon {
  font-size: 1.5rem;
  color: var(--status-success, #3fa96a);
  font-variation-settings: 'FILL' 1, 'wght' 400, 'GRAD' 0, 'opsz' 24;
  flex-shrink: 0;
}

.onboarding-connected-name {
  margin: 0.1rem 0 0;
  font-size: 0.85rem;
}

/* ── Actions row ── */
.onboarding-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.6rem;
}

.btn-ghost {
  background: none;
  border: none;
  color: var(--ink-muted);
  cursor: pointer;
  font-size: 0.9rem;
  padding: 0.5rem 0.8rem;
  border-radius: var(--radius-sm);
  transition: color 120ms ease;
}

.btn-ghost:hover {
  color: var(--ink);
}

/* ── Ranks step ── */
.onboarding-ranks {
  display: grid;
  gap: 0.6rem;
}

.onboarding-rank-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.7rem 0.9rem;
  border-radius: var(--radius-md);
  background: var(--surface-2, var(--surface));
  border: 1px solid var(--line);
  cursor: default;
}

.onboarding-rank-label {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  font-weight: 600;
  font-size: 0.92rem;
}

.onboarding-rank-label .material-symbols-rounded {
  font-size: 1.1rem;
  color: var(--ink-muted);
}

.onboarding-rank-select-wrap {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.onboarding-rank-icon {
  width: 1.5rem;
  height: 1.5rem;
  object-fit: contain;
  flex-shrink: 0;
}

.onboarding-rank-select {
  font-size: 0.9rem;
  padding: 0.3rem 0.5rem;
  border-radius: var(--radius-sm);
  border: 1px solid var(--line);
  background: var(--surface);
  color: var(--ink);
  cursor: pointer;
}
</style>
