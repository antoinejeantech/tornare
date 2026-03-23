<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { apiCall } from '../lib/api'
import overwatchLogo from '../assets/branding/overwatch-logo-gold.png'
import { getRankIcon, overwatchRanks } from '../lib/ranks'
import { getRoleIcon } from '../lib/roles'
import { useAuthStore } from '../stores/auth'
import { useAlertsStore } from '../stores/alerts'
import { useConfirm } from '../lib/confirm'
import ProfileHeroCard from '../components/profile/ProfileHeroCard.vue'
import ProfileGamesCard from '../components/profile/ProfileGamesCard.vue'
import type { AuthUser } from '../types'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const profile = ref<AuthUser | null>(null)
const loadingProfile = ref(false)
const savingProfile = ref(false)
const connectingBnet = ref(false)
const disconnectingBnet = ref(false)
const deletingAccount = ref(false)
const error = ref('')
const notice = ref('')
const profileFormTouched = ref(false)
const editingAccount = ref(false)
const editingRanks = ref(false)
const editUsername = ref('')
const editDisplayName = ref('')
const editEmail = ref('')
const editBattletag = ref('')
const editRankTank = ref('Unranked')
const editRankDps = ref('Unranked')
const editRankSupport = ref('Unranked')
const editPassword = ref('')
const editPasswordConfirm = ref('')

const alertsStore = useAlertsStore()
const confirm = useConfirm()

const profileId = computed(() => String(route.params.id || ''))
const viewerId = computed(() => String(authStore.user?.id || ''))
const viewerRole = computed(() => String(authStore.user?.role || '').trim().toLowerCase())
const isAdminViewer = computed(() => authStore.isAuthenticated && viewerRole.value === 'admin')
const canEdit = computed(() => {
  if (!authStore.isAuthenticated || !profileId.value) {
    return false
  }

  return viewerId.value === profileId.value || isAdminViewer.value
})
const profileInitial = computed(() => {
  const label = String(profile.value?.display_name || profile.value?.username || '').trim()
  return label.length > 0 ? label[0].toUpperCase() : 'A'
})
const isVerifiedProfile = computed(() => String(profile.value?.role || '').trim().toLowerCase() === 'admin')
const hasAccountChanges = computed(() => {
  if (!profile.value) {
    return false
  }

  const nextDisplayName = editDisplayName.value.trim()
  const nextUsername = editUsername.value.trim().toLowerCase()
  const nextEmail = editEmail.value.trim().toLowerCase()
  const currentUsername = String(profile.value.username || '').trim().toLowerCase()
  const currentDisplayName = String(profile.value.display_name || '').trim()
  const currentEmail = String(profile.value.email || '').trim().toLowerCase()
  const hasPasswordInput = editPassword.value.trim().length > 0 || editPasswordConfirm.value.trim().length > 0

  return (
    nextUsername !== currentUsername ||
    nextDisplayName !== currentDisplayName ||
    nextEmail !== currentEmail ||
    hasPasswordInput
  )
})

const hasOverwatchChanges = computed(() => {
  if (!profile.value) {
    return false
  }

  const nextBattletag = editBattletag.value.trim()
  const nextRankTank = editRankTank.value
  const nextRankDps = editRankDps.value
  const nextRankSupport = editRankSupport.value
  const currentBattletag = String(profile.value.battletag || '').trim()
  const currentRankTank = String(profile.value.rank_tank || 'Unranked')
  const currentRankDps = String(profile.value.rank_dps || 'Unranked')
  const currentRankSupport = String(profile.value.rank_support || 'Unranked')

  return (
    nextBattletag !== currentBattletag ||
    nextRankTank !== currentRankTank ||
    nextRankDps !== currentRankDps ||
    nextRankSupport !== currentRankSupport
  )
})

const hasProfileChanges = computed(() => hasAccountChanges.value || hasOverwatchChanges.value)

const canSaveProfile = computed(() => {
  return canEdit.value && profileFormTouched.value && hasProfileChanges.value && !savingProfile.value
})
const canSaveAccountSection = computed(() => {
  return canSaveProfile.value && editingAccount.value && hasAccountChanges.value
})
const canSaveOverwatchSection = computed(() => {
  return canSaveProfile.value && hasOverwatchChanges.value
})

const overwatchSummaryRows = computed(() => {
  if (!profile.value) {
    return []
  }

  return [
    { role: 'Tank', rank: String(profile.value.rank_tank || 'Unranked'), icon: getRankIcon(String(profile.value.rank_tank || 'Unranked')) },
    { role: 'DPS', rank: String(profile.value.rank_dps || 'Unranked'), icon: getRankIcon(String(profile.value.rank_dps || 'Unranked')) },
    { role: 'Support', rank: String(profile.value.rank_support || 'Unranked'), icon: getRankIcon(String(profile.value.rank_support || 'Unranked')) },
  ]
})

function markProfileFormTouched() {
  profileFormTouched.value = true
}

function hydrateFormFromProfile(value: AuthUser | null | undefined) {
  if (!value) {
    return
  }

  editDisplayName.value = String(value.display_name || '')
  editUsername.value = String(value.username || '')
  editEmail.value = String(value.email || '')
  editBattletag.value = String(value.battletag || '')
  editRankTank.value = String(value.rank_tank || 'Unranked')
  editRankDps.value = String(value.rank_dps || 'Unranked')
  editRankSupport.value = String(value.rank_support || 'Unranked')
  editPassword.value = ''
  editPasswordConfirm.value = ''
  profileFormTouched.value = false
}

function startEdit(section: string) {
  if (section === 'account') {
    editingAccount.value = true
  }
}

function cancelEditRanks() {
  hydrateFormFromProfile(profile.value)
  editingRanks.value = false
}

function cancelEditSection(section: string) {
  hydrateFormFromProfile(profile.value)
  if (section === 'account') {
    editingAccount.value = false
  }
}

function setError(message: string) {
  error.value = message
  notice.value = ''
}

function setNotice(message: string) {
  notice.value = message
  error.value = ''
}

async function loadProfile() {
  if (!profileId.value) {
    profile.value = null
    setError('Profile id is missing')
    return
  }

  loadingProfile.value = true
  try {
    error.value = ''
    notice.value = ''
    const response = await apiCall<AuthUser>(`/api/users/${profileId.value}`)
    profile.value = response
    hydrateFormFromProfile(response)
    editingAccount.value = false
    editingRanks.value = false
  } catch (err) {
    profile.value = null
    setError(err instanceof Error ? err.message : 'Failed to load profile')
  } finally {
    loadingProfile.value = false
  }
}

async function saveProfile() {
  if ((!canSaveAccountSection.value && !canSaveOverwatchSection.value) || !profile.value) {
    return
  }

  const nextDisplayName = editDisplayName.value.trim()
  const nextUsername = editUsername.value.trim().toLowerCase()
  const nextEmail = editEmail.value.trim().toLowerCase()
  const nextBattletag = editBattletag.value.trim()

  if (!nextUsername) {
    setError('Username is required')
    return
  }

  if (nextUsername.length < 3 || nextUsername.length > 24) {
    setError('Username must be 3-24 characters long')
    return
  }

  if (!/^[a-z0-9_]+$/.test(nextUsername)) {
    setError('Username can only use lowercase letters, numbers, and underscores')
    return
  }

  if (!nextDisplayName) {
    setError('Display name is required')
    return
  }

  if (!nextEmail || !nextEmail.includes('@')) {
    setError('A valid email is required')
    return
  }

  const nextPassword = editPassword.value.trim()
  const nextPasswordConfirm = editPasswordConfirm.value.trim()
  const hasPasswordUpdate = nextPassword.length > 0 || nextPasswordConfirm.length > 0

  if (hasPasswordUpdate) {
    if (nextPassword.length < 8) {
      setError('Password must be at least 8 characters long')
      return
    }

    if (nextPassword !== nextPasswordConfirm) {
      setError('Passwords do not match')
      return
    }
  }

  savingProfile.value = true
  try {
    const updated = await apiCall<AuthUser>(`/api/users/${profileId.value}`, {
      method: 'PUT',
      body: JSON.stringify({
        username: nextUsername,
        display_name: nextDisplayName,
        email: nextEmail,
        battletag: nextBattletag || null,
        rank_tank: editRankTank.value,
        rank_dps: editRankDps.value,
        rank_support: editRankSupport.value,
        new_password: hasPasswordUpdate ? nextPassword : null,
        new_password_confirm: hasPasswordUpdate ? nextPasswordConfirm : null,
      }),
    })
    profile.value = updated
    hydrateFormFromProfile(updated)
    if (viewerId.value === profileId.value) {
      authStore.user = updated
    }
    editingAccount.value = false
    editingRanks.value = false
    setNotice('Profile updated')
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to update profile')
  } finally {
    savingProfile.value = false
  }
}

function goToEvents() {
  router.push({ name: 'events' })
}

async function connectBnetAccount() {
  if (connectingBnet.value) return
  connectingBnet.value = true
  try {
    await authStore.connectBnetInit()
    // connectBnetInit navigates away; if it throws, land back here
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to initiate Battle.net connection')
    connectingBnet.value = false
  }
}

async function disconnectBnetAccount() {
  if (disconnectingBnet.value) return
  disconnectingBnet.value = true
  try {
    await authStore.disconnectBnet()
    await loadProfile()
    alertsStore.push({ type: 'success', message: 'Battle.net account disconnected' })
  } catch (err) {
    alertsStore.push({
      type: 'error',
      message: err instanceof Error ? err.message : 'Failed to disconnect Battle.net',
      duration: 6000,
    })
  } finally {
    disconnectingBnet.value = false
  }
}

async function deleteUserAccount() {
  if (deletingAccount.value || !profile.value) return
  const confirmed = await confirm.ask({
    title: 'Delete account',
    message: `Permanently delete ${profile.value.display_name}'s account? This cannot be undone.`,
    confirmText: 'Delete account',
    tone: 'danger',
  })
  if (!confirmed) return
  deletingAccount.value = true
  try {
    await apiCall(`/api/users/${profile.value.id}`, { method: 'DELETE' })
    alertsStore.push({ type: 'success', message: 'Account deleted' })
    router.push('/events')
  } catch (err) {
    alertsStore.push({
      type: 'error',
      message: err instanceof Error ? err.message : 'Failed to delete account',
      duration: 6000,
    })
  } finally {
    deletingAccount.value = false
  }
}

watch(
  () => route.params.id,
  () => {
    loadProfile()
  }
)

watch(
  canEdit,
  (allowed) => {
    if (allowed && profile.value) {
      hydrateFormFromProfile(profile.value)
    }

    editingAccount.value = false
    editingRanks.value = false
  }
)

onMounted(async () => {
  await loadProfile()
})
</script>

<template>
  <main class="app-shell profile-shell">
    <header class="profile-hero-header">
      <p v-if="isVerifiedProfile" class="profile-hero-eyebrow">
        <span class="material-symbols-rounded profile-hero-eyebrow-icon" aria-hidden="true">verified_user</span>
        <span>Verified Profile</span>
      </p>
      <h1 class="profile-hero-title">Profile</h1>
    </header>

    <p v-if="error" class="status status-error">{{ error }}</p>
    <p v-else-if="notice" class="status status-ok">{{ notice }}</p>

    <section v-if="loadingProfile" class="card">
      <p>Loading profile...</p>
    </section>

    <section v-else-if="profile" class="profile-layout">
      <div class="profile-column profile-column-left">
        <ProfileHeroCard
          :profile="profile"
          :can-edit="canEdit"
          :editing-account="editingAccount"
          :profile-initial="profileInitial"
          @edit-account="startEdit('account')"
        >
          <template v-if="isAdminViewer && profileId !== viewerId" #hero-actions>
            <button
              type="button"
              class="hero-icon-btn hero-icon-btn-danger"
              title="Delete account"
              :disabled="deletingAccount"
              @click="deleteUserAccount"
            >
              <span class="material-symbols-rounded" aria-hidden="true">delete</span>
              <span class="sr-only">Delete account</span>
            </button>
          </template>
          <template #account-edit>
            <form class="profile-form" @submit.prevent="saveProfile">
              <label>
                Username
                <input v-model="editUsername" placeholder="username" @input="markProfileFormTouched" />
              </label>
              <label>
                Display name
                <input v-model="editDisplayName" placeholder="Your display name" @input="markProfileFormTouched" />
              </label>
              <label>
                Email
                <input v-model="editEmail" type="email" placeholder="you@example.com" @input="markProfileFormTouched" />
              </label>
              <label>
                New password
                <input v-model="editPassword" type="password" placeholder="Leave blank to keep current password" @input="markProfileFormTouched" />
              </label>
              <label>
                Confirm new password
                <input v-model="editPasswordConfirm" type="password" placeholder="Repeat new password" @input="markProfileFormTouched" />
              </label>
              <div class="form-actions">
                <button type="submit" class="btn-primary" :disabled="!canSaveAccountSection">
                  <span class="material-symbols-rounded" aria-hidden="true">{{ savingProfile ? 'hourglass_empty' : 'check' }}</span>
                  <span>{{ savingProfile ? 'Saving...' : 'Save account' }}</span>
                </button>
                <button type="button" class="btn-secondary" :disabled="savingProfile" @click="cancelEditSection('account')">
                  <span class="material-symbols-rounded" aria-hidden="true">close</span>
                  <span>Cancel</span>
                </button>
              </div>
            </form>
          </template>
        </ProfileHeroCard>
      </div>

      <div class="profile-column profile-column-right">
        <ProfileGamesCard
          :profile="profile"
          :can-edit="canEdit"
          :overwatch-summary-rows="overwatchSummaryRows"
          :overwatch-logo="overwatchLogo"
        >
          <template #overwatch-bnet-action>
            <template v-if="canEdit">
              <p v-if="!profile.can_edit_battletag && !profile.has_password" class="bnet-no-password-warning">
                Set a password before disconnecting Battle.net, otherwise you will lose access to your account.
              </p>
              <button
                v-if="!profile.can_edit_battletag"
                type="button"
                class="battletag-link battletag-link-active"
                :disabled="disconnectingBnet || !profile.has_password"
                @click="disconnectBnetAccount"
              >
                {{ disconnectingBnet ? 'Disconnecting...' : 'Disconnect Battle.net' }}
              </button>
              <button
                v-else
                type="button"
                class="battletag-link battletag-link-active"
                :disabled="connectingBnet"
                @click="connectBnetAccount"
              >
                {{ connectingBnet ? 'Connecting...' : 'Connect Battle.net Account' }}
              </button>
            </template>
          </template>

          <template v-if="canEdit" #overwatch-ranks>
            <template v-if="!editingRanks">
              <div class="rank-tiles-wrap">
                <div class="rank-tiles-header">
                  <button class="rank-edit-btn" type="button" @click="editingRanks = true">
                    <span class="material-symbols-rounded" aria-hidden="true">edit</span>
                    <span>Edit ranks</span>
                  </button>
                </div>
                <div class="rank-tile-grid">
                  <article v-for="entry in overwatchSummaryRows" :key="entry.role" class="rank-tile">
                    <p class="rank-role">
                      <span>{{ entry.role }}</span>
                      <span class="material-symbols-rounded rank-role-icon" aria-hidden="true">{{ getRoleIcon(entry.role) }}</span>
                    </p>
                    <p class="rank-value">
                      <img class="rank-icon" :src="entry.icon" :alt="`${entry.rank} rank`" />
                      <span>{{ entry.rank }}</span>
                    </p>
                  </article>
                </div>
              </div>
            </template>
            <form v-else class="profile-form" @submit.prevent="saveProfile">
              <div class="profile-ranks-grid">
                <label>
                  Tank rank
                  <select v-model="editRankTank" @change="markProfileFormTouched">
                    <option v-for="rank in overwatchRanks" :key="`tank-${rank}`" :value="rank">{{ rank }}</option>
                  </select>
                </label>
                <label>
                  DPS rank
                  <select v-model="editRankDps" @change="markProfileFormTouched">
                    <option v-for="rank in overwatchRanks" :key="`dps-${rank}`" :value="rank">{{ rank }}</option>
                  </select>
                </label>
                <label>
                  Support rank
                  <select v-model="editRankSupport" @change="markProfileFormTouched">
                    <option v-for="rank in overwatchRanks" :key="`support-${rank}`" :value="rank">{{ rank }}</option>
                  </select>
                </label>
              </div>
              <div class="form-actions">
                <button type="submit" class="btn-primary" :disabled="!canSaveOverwatchSection">
                  <span class="material-symbols-rounded" aria-hidden="true">{{ savingProfile ? 'hourglass_empty' : 'check' }}</span>
                  <span>{{ savingProfile ? 'Saving...' : 'Save ranks' }}</span>
                </button>
                <button type="button" class="btn-secondary" :disabled="savingProfile" @click="cancelEditRanks">
                  <span class="material-symbols-rounded" aria-hidden="true">close</span>
                  <span>Cancel</span>
                </button>
              </div>
            </form>
          </template>
        </ProfileGamesCard>
      </div>
    </section>

    <section v-else class="card">
      <h2>Profile not found</h2>
      <p class="muted">The user may not exist.</p>
      <button class="btn-secondary" @click="goToEvents">Back to events</button>
    </section>
  </main>
</template>

<style scoped>
.profile-shell {
  max-width: 1260px;
  width: min(96vw, 1260px);
}

.profile-hero-header {
  display: grid;
  gap: 0.18rem;
}

.profile-hero-eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 0.28rem;
  margin: 0;
  color: var(--brand-1);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  font-size: 0.68rem;
  font-weight: 700;
}

.profile-hero-eyebrow-icon {
  color: color-mix(in srgb, var(--brand-1) 90%, #ffe7aa 10%);
  font-size: 0.9rem;
  font-variation-settings: 'FILL' 1, 'wght' 700, 'GRAD' 0, 'opsz' 20;
}

.profile-hero-title {
  margin: 0;
  text-transform: uppercase;
  font-size: clamp(2.05rem, 1.6rem + 1.75vw, 2.9rem);
  line-height: 1;
  letter-spacing: 0.015em;
}

.profile-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 0.9rem;
  align-items: start;
}

.profile-layout :deep(.card) {
  background: transparent;
}

.profile-column {
  min-width: 0;
  animation: profile-card-in 320ms ease-out both;
}

.profile-column-right {
  animation-delay: 80ms;
}

.profile-form {
  display: grid;
  gap: 0.7rem;
  border: 1px solid color-mix(in srgb, var(--line) 82%, var(--brand-1) 18%);
  border-radius: var(--radius-md);
  padding: 0.85rem;
}

.profile-form label {
  display: grid;
  gap: 0.3rem;
}

/* BNet slot buttons rendered inside ProfileGamesCard */
:deep(.battletag-link) {
  justify-self: center;
  background: none;
  border: none;
  padding: 0;
  font-size: 0.86rem;
  font-weight: 700;
  text-decoration: none;
  color: color-mix(in srgb, var(--ink-muted) 82%, transparent 18%);
  cursor: not-allowed;
}

:deep(.battletag-link-active) {
  color: color-mix(in srgb, var(--brand-1) 95%, #ffefbf 5%);
  cursor: pointer;
}

:deep(.battletag-link-active:disabled) {
  opacity: 0.45;
  cursor: not-allowed;
  text-decoration: none;
}

.bnet-no-password-warning {
  margin: 0 0 0.45rem;
  font-size: 0.82rem;
  color: var(--status-warning, #c97c00);
}

:deep(.hero-icon-btn-danger) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.1rem;
  height: 2.1rem;
  border-radius: var(--radius-sm);
  border: 1px solid color-mix(in srgb, var(--status-error, #c0392b) 40%, var(--line) 60%);
  background: transparent;
  color: var(--status-error, #c0392b);
  cursor: pointer;
  transition: background 120ms, color 120ms, border-color 120ms;
}

:deep(.hero-icon-btn-danger .material-symbols-rounded) {
  font-size: 1.1rem;
  font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 20;
}

:deep(.hero-icon-btn-danger:hover:not(:disabled)) {
  background: color-mix(in srgb, var(--status-error, #c0392b) 12%, transparent);
  border-color: var(--status-error, #c0392b);
}

:deep(.hero-icon-btn-danger:disabled) {
  opacity: 0.45;
  cursor: not-allowed;
}

:deep(.battletag-link-active:hover) {
  text-decoration: underline;
}

.profile-note {
  margin: 0;
  font-size: 0.88rem;
}

.profile-ranks-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.72rem;
}

.profile-note-title {
  margin: 0;
  color: var(--ink-2);
  font-size: 0.74rem;
  text-transform: uppercase;
  letter-spacing: 0.11em;
  font-weight: 700;
}

.profile-static-value {
  margin: 0;
  border: 1px solid color-mix(in srgb, var(--line) 82%, var(--brand-1) 18%);
  border-radius: var(--radius-md);
  padding: 0.55rem 0.7rem;
  font-weight: 600;
}

.form-actions .material-symbols-rounded {
  font-size: 1rem;
  font-variation-settings: 'FILL' 0, 'wght' 500, 'GRAD' 0, 'opsz' 20;
}

@keyframes profile-card-in {
  from {
    opacity: 0;
    transform: translateY(9px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Rank tiles — scoped CSS from ProfileGamesCard doesn't reach slot content, so we re-declare here */
.rank-tiles-wrap {
  display: grid;
  gap: 0.5rem;
}

.rank-tiles-header {
  display: flex;
  justify-content: flex-end;
}

.rank-tile-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.72rem;
}

.rank-tile {
  display: grid;
  gap: 0.46rem;
  padding: 0.84rem 0.84rem 0.72rem;
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--line) 26%, transparent 74%);
  position: relative;
}

.rank-role,
.rank-value {
  margin: 0;
}

.rank-role {
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.72rem;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 700;
}

.rank-role-icon {
  font-size: 0.92rem;
  color: color-mix(in srgb, var(--brand-1) 92%, #ffe7aa 8%);
}

.rank-value {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  font-weight: 760;
  color: var(--ink-1);
}

.rank-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.rank-edit-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.36rem;
  align-self: start;
  justify-self: start;
  background: none;
  border: 1px solid color-mix(in srgb, var(--line) 50%, transparent 50%);
  border-radius: var(--radius-sm);
  padding: 0.32rem 0.72rem;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--ink-muted);
  cursor: pointer;
  transition: color 120ms, border-color 120ms;
}

.rank-edit-btn:hover {
  color: var(--ink-1);
  border-color: color-mix(in srgb, var(--line) 80%, transparent 20%);
}

.rank-edit-btn .material-symbols-rounded {
  font-size: 0.92rem;
  font-variation-settings: 'FILL' 0, 'wght' 450, 'GRAD' 0, 'opsz' 20;
}

@media (max-width: 980px) {
  .profile-layout {
    grid-template-columns: 1fr;
  }

  .profile-ranks-grid {
    grid-template-columns: 1fr;
  }

  .rank-tile-grid {
    grid-template-columns: 1fr;
  }
}
</style>
