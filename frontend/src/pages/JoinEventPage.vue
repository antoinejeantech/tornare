<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { overwatchRanks } from '../lib/ranks'
import { formatEventStartDate } from '../lib/dates'
import { useEventStore } from '../stores/event'
import { useAuthStore } from '../stores/auth'
import InlineArrowLink from '../components/ui/InlineArrowLink.vue'
import AppBadge from '../components/ui/AppBadge.vue'
import BnetIcon from '../components/ui/BnetIcon.vue'
import DiscordIcon from '../components/ui/DiscordIcon.vue'
import type { PublicSignupInfo } from '../types'

const route = useRoute()
const router = useRouter()
const eventStore = useEventStore()
const authStore = useAuthStore()

const loading = ref(false)
const submitting = ref(false)
const error = ref('')
const notice = ref('')
const signupInfo = ref<PublicSignupInfo | null>(null)

const playerName = ref('')
const playerRoles = ref<Array<{ role: string; rank: string }>>([{ role: '', rank: '' }])
const playerDiscord = ref('')
const playerBattletag = ref('')

function addRole() {
  if (playerRoles.value.length < 3) {
    playerRoles.value.push({ role: '', rank: '' })
  }
}

function removeRole(index: number) {
  if (playerRoles.value.length > 1) {
    playerRoles.value.splice(index, 1)
  }
}

const usedRoles = computed(() => playerRoles.value.map(rp => rp.role))

function isRoleTaken(role: string, currentIndex: number): boolean {
  if (!role) return false
  return usedRoles.value.some((r, i) => i !== currentIndex && r === role)
}

const availableRolesForNewSlot = computed(() => {
  const all = ['Tank', 'DPS', 'Support']
  return all.filter(r => !usedRoles.value.includes(r))
})

const signupToken = computed(() => String(route.params.token || ''))
const MAX_SIGNUP_REQUESTS_PER_EVENT = 99

const rosterFull = computed(() => {
  if (!signupInfo.value) {
    return false
  }
  return signupInfo.value.current_players >= signupInfo.value.max_players
})

const signupRequestsFull = computed(() => {
  if (!signupInfo.value) {
    return false
  }
  return signupInfo.value.current_signup_requests >= MAX_SIGNUP_REQUESTS_PER_EVENT
})

const formattedStartDate = computed(() => {
  if (!signupInfo.value?.start_date) {
    return 'TBA'
  }
  return formatEventStartDate(signupInfo.value.start_date) || 'TBA'
})

const canSubmit = computed(() => {
  return (
    signupToken.value.length > 0 &&
    playerName.value.trim().length > 0 &&
    playerRoles.value.length > 0 &&
    playerRoles.value.every(rp => rp.role.trim().length > 0 && rp.rank.trim().length > 0) &&
    !submitting.value &&
    !signupRequestsFull.value
  )
})

function setError(message: string) {
  error.value = message
  notice.value = ''
}

function setNotice(message: string) {
  notice.value = message
  error.value = ''
}

async function loadSignupInfo() {
  if (!signupToken.value) {
    setError('Invalid signup link')
    return
  }

  loading.value = true
  try {
    signupInfo.value = await eventStore.fetchPublicSignupInfo(signupToken.value)
  } catch (err) {
    signupInfo.value = null
    setError(err instanceof Error ? err.message : 'Failed to load signup link')
  } finally {
    loading.value = false
  }
}

async function submitRequest() {
  if (!canSubmit.value) {
    return
  }

  submitting.value = true
  try {
    await eventStore.submitPublicSignupRequest(signupToken.value, {
      name: playerName.value.trim(),
      roles: playerRoles.value,
      discord_username: playerDiscord.value.trim() || null,
      battletag: playerBattletag.value.trim() || null,
    })

    const destinationEventId = signupInfo.value?.event_id
    if (destinationEventId) {
      await router.push({ name: 'event', params: { id: String(destinationEventId) } })
      return
    }

    if (signupInfo.value) {
      signupInfo.value = {
        ...signupInfo.value,
        current_signup_requests: signupInfo.value.current_signup_requests + 1,
      }
    }

    playerName.value = ''
    playerRoles.value = [{ role: '', rank: '' }]
    setNotice('Request sent. The event owner will review it soon.')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to submit request')
  } finally {
    submitting.value = false
  }
}

function prefillFromAuth() {
  const user = authStore.user
  if (!user) return

  if (!playerName.value) {
    playerName.value = user.display_name
  }

  // Only prefill manual fields when the account isn't already linked
  const entries: Array<{ role: string; rank: string }> = []
  if (user.rank_tank && user.rank_tank !== 'Unranked') entries.push({ role: 'Tank', rank: user.rank_tank })
  if (user.rank_dps && user.rank_dps !== 'Unranked') entries.push({ role: 'DPS', rank: user.rank_dps })
  if (user.rank_support && user.rank_support !== 'Unranked') entries.push({ role: 'Support', rank: user.rank_support })
  if (entries.length > 0) playerRoles.value = entries
}

onMounted(async () => {
  await loadSignupInfo()
  prefillFromAuth()
})
</script>

<template>
  <main class="app-shell join-shell">
    <header class="page-header">
      <InlineArrowLink class="join-back-link" to="/events" label="Return to events list" arrow-side="left" />
      <h1 class="page-title">Join Event</h1>
    </header>

    <section class="card join-card">
      <p v-if="loading" class="join-loading">Loading signup page...</p>

      <template v-else-if="signupInfo">
        <div class="join-head">
          <AppBadge
            label="Public Signup"
            radius="pill"
            bg="color-mix(in srgb, var(--primary-700) 22%, transparent 78%)"
            color="var(--primary-300)"
            border="color-mix(in srgb, var(--primary-500) 52%, var(--line) 48%)"
            style="justify-self: start"
          />
          <h2 class="join-event-title">{{ signupInfo.event_name }}</h2>
          <div class="join-event-meta-row">
            <span class="join-event-meta-item">
              <span class="material-symbols-rounded" aria-hidden="true">trophy</span>
              <span>{{ signupInfo.event_type }}</span>
            </span>
            <span class="material-symbols-rounded join-event-meta-dot" aria-hidden="true">fiber_manual_record</span>
            <span class="join-event-meta-item join-event-meta-item-format">
              <span>{{ signupInfo.format || '5v5' }}</span>
            </span>
          </div>
          <p v-if="signupInfo.event_description" class="join-event-description">{{ signupInfo.event_description }}</p>
          <template v-if="rosterFull && !signupRequestsFull">
            <div class="join-full-state">
              <span class="material-symbols-rounded join-full-state-icon" aria-hidden="true">info</span>
              <div class="join-full-state-copy">
                <p class="join-full-state-title">Event is currently full</p>
                <p class="join-full-state-text">You can still send a request to join. The tournament organizer may increase slots or approve pending requests manually.</p>
              </div>
            </div>
            <div class="join-separator" aria-hidden="true"></div>
          </template>
          <div class="join-stats">
            <article class="join-stat-card">
              <span class="join-stat-icon-wrap" aria-hidden="true">
                <span class="material-symbols-rounded join-stat-icon">groups</span>
              </span>
              <div class="join-stat-copy">
                <span class="join-stat-label">Registered players</span>
                <strong class="join-stat-value">
                  {{ signupInfo.current_players }}/{{ signupInfo.max_players }}
                  <span v-if="rosterFull">(Full)</span>
                </strong>
              </div>
            </article>
            <article class="join-stat-card">
              <span class="join-stat-icon-wrap" aria-hidden="true">
                <span class="material-symbols-rounded join-stat-icon">calendar_month</span>
              </span>
              <div class="join-stat-copy">
                <span class="join-stat-label">{{ signupInfo.event_type === 'TOURNEY' ? 'Tournament start' : 'Event start' }}</span>
                <strong class="join-stat-value">{{ formattedStartDate }}</strong>
              </div>
            </article>
          </div>
        </div>

        <p v-if="error" class="status status-error">{{ error }}</p>
        <p v-else-if="notice" class="status status-ok">{{ notice }}</p>

        <p v-if="signupRequestsFull" class="status status-blocked">Signup is currently unavailable because this event reached the request limit.</p>

        <div v-if="!authStore.isAuthenticated" class="join-auth-hint">
          <span class="material-symbols-rounded" aria-hidden="true">account_circle</span>
          <p><RouterLink to="/auth">Sign in</RouterLink> to automatically prefill your name and ranks.</p>
        </div>

        <div v-else class="join-auth-banner">
          <span class="material-symbols-rounded" aria-hidden="true">verified_user</span>
          <p>Signing up as <strong>{{ authStore.user?.display_name }}</strong> (@{{ authStore.user?.username }}). Your request will be linked to your account.</p>
        </div>

        <form class="join-form" @submit.prevent="submitRequest">
          <label class="join-field join-field-full">
            <span class="join-field-label">YOUR DISPLAY NAME</span>
            <div class="join-input-leading-icon">
              <span class="material-symbols-rounded" aria-hidden="true">sports_esports</span>
              <input v-model="playerName" placeholder="Your display or nickname" />
            </div>
          </label>

          <!-- Discord -->
          <div class="join-field">
            <span class="join-field-label">DISCORD <span class="join-field-optional">optional</span></span>
            <div v-if="authStore.user?.discord_username" class="join-verified-chip">
              <DiscordIcon class="join-verified-icon" />
              <span class="join-verified-value">{{ authStore.user.discord_username }}</span>
              <span class="join-verified-badge">
                <span class="material-symbols-rounded" aria-hidden="true">verified</span>
                Verified
              </span>
            </div>
            <div v-else class="join-input-leading-icon">
              <DiscordIcon class="join-platform-icon" />
              <input v-model="playerDiscord" placeholder="Your Discord username" maxlength="100" />
            </div>
          </div>

          <!-- Battle.net -->
          <div class="join-field">
            <span class="join-field-label">BATTLE.NET <span class="join-field-optional">optional</span></span>
            <div v-if="authStore.user?.battletag" class="join-verified-chip">
              <BnetIcon class="join-verified-icon join-verified-icon--bnet" />
              <span class="join-verified-value">{{ authStore.user.battletag }}</span>
              <span class="join-verified-badge join-verified-badge--bnet">
                <span class="material-symbols-rounded" aria-hidden="true">verified</span>
                Verified
              </span>
            </div>
            <div v-else class="join-input-leading-icon">
              <BnetIcon class="join-platform-icon join-platform-icon--bnet" />
              <input v-model="playerBattletag" placeholder="YourName#1234" maxlength="100" />
            </div>
          </div>

          <div class="join-field-full join-roles-section">
            <span class="join-roles-label">ROLE PREFERENCES</span>
            <ul class="join-roles-list">
              <li
                v-for="(entry, index) in playerRoles"
                :key="index"
                class="join-role-row"
                :class="{ 'join-role-row--removable': playerRoles.length > 1 }"
              >
                <label class="join-field">
                  <span class="join-role-field-lbl">Role<span v-if="index === 0" class="join-role-pref-hint">preferred</span></span>
                  <select v-model="entry.role">
                    <option value="" disabled hidden></option>
                    <option value="Tank" :disabled="isRoleTaken('Tank', index)">Tank</option>
                    <option value="DPS" :disabled="isRoleTaken('DPS', index)">DPS</option>
                    <option value="Support" :disabled="isRoleTaken('Support', index)">Support</option>
                  </select>
                </label>
                <label class="join-field">
                  Rank
                  <select v-model="entry.rank">
                    <option value="" disabled hidden></option>
                    <option v-for="rank in overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
                  </select>
                </label>
                <div v-if="playerRoles.length > 1" class="join-role-remove-col">
                  <span class="join-role-remove-spacer" aria-hidden="true">Role</span>
                  <button
                    type="button"
                    class="join-role-remove"
                    :aria-label="`Remove role preference ${index + 1}`"
                    @click="removeRole(index)"
                  >
                    <span class="material-symbols-rounded" aria-hidden="true">delete</span>
                  </button>
                </div>
              </li>
            </ul>
            <button
              v-if="playerRoles.length < 3 && availableRolesForNewSlot.length > 0"
              type="button"
              class="join-add-role"
              @click="addRole"
            >
              <span class="material-symbols-rounded" aria-hidden="true">add</span>
              Add role
            </button>
          </div>

          <div class="join-actions">
            <button type="submit" class="btn-primary" :disabled="!canSubmit">
              {{ submitting ? 'Submitting...' : 'Request to join' }}
            </button>
            <p class="join-actions-note">By requesting to join, you agree to our Tournament Fair Play Guidelines.</p>
          </div>
        </form>
      </template>

      <template v-else>
        <div class="join-unavailable">
          <h2>Signup link unavailable</h2>
          <p class="muted">This link may be invalid or has expired.</p>
        </div>
      </template>
    </section>
  </main>
</template>

<style scoped>
.join-shell {
  width: min(95vw, 940px);
  grid-template-columns: minmax(0, 1fr);
}

.page-header {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: flex-start;
  gap: 0.45rem;
  width: min(100%, 860px);
  margin-inline: auto;
}

.page-title {
  margin: 0;
  font-size: clamp(2rem, 2.8vw + 1rem, 3rem);
  font-weight: 800;
  letter-spacing: -0.02em;
  line-height: 1.05;
  width: 100%;
  text-align: center;
}

.join-back-link {
  display: inline-flex;
  align-items: center;
  gap: 0.36rem;
  color: var(--ink-2);
  text-decoration: none;
  font-weight: 600;
}

.join-back-link :deep(svg) {
  width: 0.9rem;
  height: 0.9rem;
}

.join-card {
  --join-card-pad: clamp(1.35rem, 2.6vw, 2rem);
  width: min(100%, 860px);
  margin-inline: auto;
  padding: clamp(1.35rem, 2.4vw, 2rem) var(--join-card-pad);
  display: grid;
  gap: 1rem;
  border: 1px solid var(--surface-card-border);
  background: var(--card);
  box-shadow: none;
}

.join-head {
  display: grid;
  gap: 0.4rem;
  margin-bottom: 0.25rem;
}

.join-event-title {
  margin: 0;
  font-size: clamp(1.35rem, 1.8vw + 0.9rem, 1.95rem);
  font-weight: 800;
}

.join-event-meta-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.9rem;
  margin: 0.15rem 0 0.3rem;
}

.join-event-meta-item {
  display: inline-flex;
  align-items: center;
  gap: 0.34rem;
  color: var(--primary-300);
  font-weight: 700;
  font-size: 0.88rem;
}

.join-event-meta-item-format {
  gap: 0;
}

.join-event-meta-item .material-symbols-rounded {
  font-size: 1.02rem;
}

.join-event-meta-dot {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: center;
  line-height: 1;
  font-size: 0.5rem;
  color: var(--primary-300);
  transform: translateY(1px);
  font-variation-settings: 'FILL' 1, 'wght' 700, 'GRAD' 0, 'opsz' 24;
}

.join-event-description {
  margin: 0.12rem 0 0.45rem;
  max-width: 66ch;
  line-height: 1.45;
  color: var(--ink-1);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.join-stats {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.72rem;
  margin-bottom: 0.9rem;
}

.join-stat-card {
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: center;
  gap: 0.72rem;
  border: 1px solid var(--surface-card-border);
  border-radius: 18px;
  padding: 0.74rem 0.86rem;
  background: var(--surface-card-bg);
  color: var(--ink-1);
}

.join-stat-icon-wrap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 2.8rem;
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--line-strong) 74%, var(--line) 26%);
  background: color-mix(in srgb, var(--bg-1) 66%, var(--card) 34%);
}

.join-stat-icon {
  font-size: 1.22rem;
  color: var(--primary-300);
}

.join-stat-copy {
  display: grid;
  gap: 0.12rem;
}

.join-stat-label {
  font-size: 0.7rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: color-mix(in srgb, var(--ink-2) 88%, white 12%);
  font-weight: 700;
}

.join-stat-value {
  font-size: 1.02rem;
  letter-spacing: 0;
  line-height: 1.2;
}

.status-blocked {
  color: var(--ink-1);
  background: color-mix(in srgb, var(--meta-bg) 28%, var(--card) 72%);
  border-color: color-mix(in srgb, var(--meta-ink) 26%, var(--line) 74%);
}

.status-blocked-soft {
  color: color-mix(in srgb, var(--warn) 54%, var(--ink-1) 46%);
  background: color-mix(in srgb, var(--warn) 12%, var(--card) 88%);
  border-color: color-mix(in srgb, var(--warn) 36%, var(--line) 64%);
}

.join-full-state {
  width: 100%;
  margin: 0.75rem 0;
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: start;
  gap: 0.56rem;
  border: 1px solid color-mix(in srgb, var(--line-strong) 78%, var(--line) 22%);
  border-radius: var(--radius-sm);
  background: transparent;
  padding: 0.95rem 0.72rem;
}

.join-full-state-icon {
  margin-top: 0.03rem;
  font-size: 1.3rem;
  color: var(--ink-1);
}

.join-full-state-copy {
  display: grid;
  gap: 0.14rem;
  min-width: 0;
}

.join-full-state-title {
  margin: 0;
  color: white;
  font-size: 0.98rem;
  font-weight: 800;
  line-height: 1.2;
}

.join-full-state-text {
  margin: 0;
  color: var(--ink-1);
  line-height: 1.4;
}

.join-separator {
  height: 1px;
  width: calc(100% + (var(--join-card-pad) * 2));
  margin: 1.05rem 0 1.05rem calc(var(--join-card-pad) * -1);
  background: color-mix(in srgb, var(--line) 72%, transparent);
}

.join-auth-hint {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  padding: 0.65rem 0.9rem;
  background: color-mix(in srgb, var(--primary-700) 14%, transparent 86%);
  border: 1px solid color-mix(in srgb, var(--primary-500) 38%, var(--line) 62%);
  border-radius: 8px;
  color: var(--primary-300);
  font-size: 0.875rem;
}

.join-auth-hint .material-symbols-rounded {
  flex-shrink: 0;
  font-size: 1.15rem;
  opacity: 0.85;
}

.join-auth-hint p {
  margin: 0;
}

.join-auth-hint a {
  color: inherit;
  text-underline-offset: 2px;
}

.join-auth-banner {
  display: flex;
  align-items: flex-start;
  gap: 0.65rem;
  padding: 0.65rem 0.9rem;
  background: color-mix(in srgb, #5865f2 12%, transparent 88%);
  border: 1px solid color-mix(in srgb, #5865f2 45%, var(--line) 55%);
  border-radius: 8px;
  color: color-mix(in srgb, #a5aaff 90%, white 10%);
  font-size: 0.875rem;
}

.join-auth-banner .material-symbols-rounded {
  flex-shrink: 0;
  font-size: 1.15rem;
  margin-top: 1px;
  opacity: 0.9;
}

.join-auth-banner p {
  margin: 0;
}

.join-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 1rem;
}

.join-roles-section {
  display: grid;
  gap: 0;
  margin-bottom: 0.75rem;
}

.join-roles-label {
  font-size: 0.76rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  margin-bottom: 0.5rem;
}

.join-roles-list {
  list-style: none;
  margin: 0;
  margin-bottom: 0.55rem;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.join-role-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  align-items: end;
  gap: 0.72rem;
}

.join-role-row--removable {
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
}

.join-role-remove-col {
  display: grid;
  gap: 0.32rem;
  align-self: end;
}

.join-role-remove-spacer {
  visibility: hidden;
  font-size: 0.76rem;
  font-weight: 700;
  line-height: 1.2;
  pointer-events: none;
  user-select: none;
}

.join-role-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  padding: 0.3rem;
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--ink-2);
  transition: color 0.14s, background 0.14s;
  min-height: 2.12rem;
}

.join-role-remove:hover {
  color: var(--danger, #e05c5c);
  background: color-mix(in srgb, var(--danger, #e05c5c) 10%, transparent 90%);
}

.join-role-remove .material-symbols-rounded {
  font-size: 1.1rem;
}

.join-add-role {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: center;
  gap: 0.3rem;
  background: color-mix(in srgb, var(--bg-1) 60%, var(--card) 40%);
  border: 1px solid var(--line);
  color: var(--ink-2);
  border-radius: var(--radius-sm);
  padding: 0.4rem 0.65rem;
  font-size: 0.76rem;
  font-weight: 600;
  cursor: pointer;
  transition: border-color 0.14s, color 0.14s;
}

.join-add-role:hover {
  border-color: var(--primary-500);
  color: var(--primary-300);
}

.join-add-role .material-symbols-rounded {
  font-size: 1rem;
}

.join-field {
  display: grid;
  gap: 0.32rem;
}

.join-input-leading-icon {
  position: relative;
  display: flex;
  align-items: center;
}

.join-input-leading-icon .material-symbols-rounded {
  position: absolute;
  left: 0.72rem;
  font-size: 1rem;
  color: var(--ink-muted);
  pointer-events: none;
}

.join-input-leading-icon input {
  width: 100%;
  padding-left: 2.15rem;
}

.join-form input::placeholder,
.join-form textarea::placeholder {
  color: color-mix(in srgb, var(--ink-muted) 70%, var(--bg-0) 30%);
}

.join-form :is(input, select, textarea) {
  background: var(--card);
}

.join-field-full {
  grid-column: 1 / -1;
}

.join-actions {
  grid-column: 1 / -1;
  display: block;
}

.join-actions .btn-primary {
  width: 100%;
  min-width: 0;
  font-weight: 800;
}

.join-actions-note {
  margin: 0.9rem 0 1rem;
  text-align: center;
  color: white;
  font-size: 0.76rem;
  line-height: 1.35;
}

.join-loading,
.join-unavailable {
  margin: 0;
}

.join-unavailable {
  display: grid;
  gap: 0.45rem;
}

.join-unavailable h2,
.join-unavailable p {
  margin: 0;
}

@media (max-width: 700px) {
  .join-stats {
    grid-template-columns: 1fr;
  }

  .join-form {
    grid-template-columns: 1fr;
  }

  .join-actions .btn-primary {
    width: 100%;
    min-width: 0;
  }
}

.join-role-field-lbl {
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
}

.join-role-pref-hint {
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--primary-300);
  background: color-mix(in srgb, var(--primary-700) 22%, var(--card) 78%);
  border: 1px solid color-mix(in srgb, var(--primary-500) 38%, var(--line) 62%);
  border-radius: var(--radius-pill);
  padding: 0.06rem 0.38rem;
}

.join-field-label {
  font-size: 0.76rem;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.join-field-optional {
  font-size: 0.65rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--ink-muted);
  margin-left: 0.3rem;
}

.join-platform-icon {
  position: absolute;
  left: 0.72rem;
  width: 1rem;
  height: 1rem;
  fill: var(--ink-muted);
  pointer-events: none;
  flex-shrink: 0;
}

.join-platform-icon--bnet {
  fill: color-mix(in srgb, #148eff 70%, var(--ink-muted) 30%);
}

.join-verified-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.48rem 0.72rem;
  border-radius: var(--radius-sm);
  border: 1px solid color-mix(in srgb, #5865f2 45%, var(--line) 55%);
  background: color-mix(in srgb, #5865f2 8%, transparent 92%);
  min-height: 2.6rem;
}

.join-verified-icon {
  width: 1rem;
  height: 1rem;
  fill: color-mix(in srgb, #adb3ff 90%, white 10%);
  flex-shrink: 0;
}

.join-verified-icon--bnet {
  fill: color-mix(in srgb, #74bbff 88%, white 12%);
}

.join-verified-value {
  font-size: 0.88rem;
  font-weight: 600;
  color: var(--ink-1);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.join-verified-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.18rem;
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: color-mix(in srgb, #adb3ff 90%, white 10%);
  background: color-mix(in srgb, #5865f2 18%, transparent 82%);
  border: 1px solid color-mix(in srgb, #5865f2 45%, var(--line) 55%);
  border-radius: var(--radius-pill);
  padding: 0.1rem 0.38rem 0.1rem 0.26rem;
  flex-shrink: 0;
  white-space: nowrap;
}

.join-verified-badge--bnet {
  color: color-mix(in srgb, #74bbff 88%, white 12%);
  background: color-mix(in srgb, #148eff 14%, transparent 86%);
  border-color: color-mix(in srgb, #148eff 45%, var(--line) 55%);
}

.join-verified-badge .material-symbols-rounded {
  font-size: 0.78rem;
  line-height: 1;
  font-variation-settings: 'FILL' 1, 'wght' 700, 'GRAD' 0, 'opsz' 24;
}
</style>
