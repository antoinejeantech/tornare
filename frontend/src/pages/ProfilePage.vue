<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { apiCall } from '../lib/api'
import overwatchLogo from '../assets/branding/overwatch-logo-gold.png'
import battlenetLogo from '../assets/branding/bnet-logo.png'
import { getRankIcon, overwatchRanks } from '../lib/ranks'
import { getRoleIcon } from '../lib/roles'
import { useAuthStore } from '../stores/auth'
import { useAlertsStore } from '../stores/alerts'
import { useConfirm } from '../composables/confirm'
import ProfileHeroCard from '../components/profile/ProfileHeroCard.vue'
import ProfileGamesCard from '../components/profile/ProfileGamesCard.vue'
import EventListItem from '../components/events/EventListItem.vue'
import InlineArrowLink from '../components/ui/InlineArrowLink.vue'
import type { AuthUser, Event } from '../types'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const profile = ref<AuthUser | null>(null)
const loadingProfile = ref(false)
const savingProfile = ref(false)
const connectingBnet = ref(false)
const disconnectingBnet = ref(false)
const connectingDiscord = ref(false)
const disconnectingDiscord = ref(false)
const connectingOAuth = ref<'bnet' | 'discord' | null>(null)
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

const userEvents = ref<Event[]>([])
const loadingUserEvents = ref(false)

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

  const nextDisplayName = String(editDisplayName.value ?? '').trim()
  const nextUsername = String(editUsername.value ?? '').trim().toLowerCase()
  const nextEmail = String(editEmail.value ?? '').trim().toLowerCase()
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

function startEdit(section: 'account' | 'ranks') {
  if (section === 'account') {
    editingRanks.value = false
    editingAccount.value = true
    return
  }

  editingAccount.value = false
  editingRanks.value = true
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

  const nextDisplayName = String(editDisplayName.value ?? '').trim()
  const nextUsername = String(editUsername.value ?? '').trim().toLowerCase()
  const nextEmail = String(editEmail.value ?? '').trim().toLowerCase()
  const nextBattletag = String(editBattletag.value ?? '').trim()

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

  if (!nextEmail || !/@/.test(nextEmail)) {
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
  connectingOAuth.value = 'bnet'
  sessionStorage.setItem('oauth_return_path', window.location.pathname)
  try {
    await authStore.connectBnetInit()
    // connectBnetInit normally navigates away immediately.
  } catch (err) {
    connectingOAuth.value = null
    setError(err instanceof Error ? err.message : 'Failed to initiate Battle.net connection')
  } finally {
    connectingBnet.value = false
  }
}

async function disconnectBnetAccount() {
  if (disconnectingBnet.value) return
  const confirmed = await confirm.ask({
    title: 'Disconnect Battle.net',
    message: 'Are you sure you want to disconnect your Battle.net account?',
    confirmText: 'Disconnect',
    tone: 'danger',
  })
  if (!confirmed) return
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

async function connectDiscordAccount() {
  if (connectingDiscord.value) return
  connectingDiscord.value = true
  connectingOAuth.value = 'discord'
  sessionStorage.setItem('oauth_return_path', window.location.pathname)
  try {
    await authStore.connectDiscordInit()
    // connectDiscordInit normally navigates away immediately.
  } catch (err) {
    connectingOAuth.value = null
    setError(err instanceof Error ? err.message : 'Failed to initiate Discord connection')
  } finally {
    connectingDiscord.value = false
  }
}

async function disconnectDiscordAccount() {
  if (disconnectingDiscord.value) return
  const confirmed = await confirm.ask({
    title: 'Disconnect Discord',
    message: 'Are you sure you want to disconnect your Discord account?',
    confirmText: 'Disconnect',
    tone: 'danger',
  })
  if (!confirmed) return
  disconnectingDiscord.value = true
  try {
    await authStore.disconnectDiscord()
    await loadProfile()
    alertsStore.push({ type: 'success', message: 'Discord account disconnected' })
  } catch (err) {
    alertsStore.push({
      type: 'error',
      message: err instanceof Error ? err.message : 'Failed to disconnect Discord',
      duration: 6000,
    })
  } finally {
    disconnectingDiscord.value = false
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
  loadUserEvents()
})

async function loadUserEvents() {
  if (!profileId.value) return
  loadingUserEvents.value = true
  try {
    const res = await apiCall<{ items: Event[]; total: number }>(
      `/api/events?owner=${encodeURIComponent(profileId.value)}&per_page=5&sort=newest`
    )
    userEvents.value = res?.items ?? []
  } catch {
    userEvents.value = []
  } finally {
    loadingUserEvents.value = false
  }
}
</script>

<template>
  <main class="app-shell profile-shell">
    <Teleport to="body">
      <div v-if="connectingOAuth" class="oauth-redirect-overlay" aria-live="polite">
        <div class="oauth-redirect-box">
          <span class="oauth-redirect-spinner" aria-hidden="true"></span>
          <p>Redirecting to {{ connectingOAuth === 'discord' ? 'Discord' : 'Battle.net' }}…</p>
        </div>
      </div>
    </Teleport>
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
                  <span class="material-symbols-rounded" aria-hidden="true">{{ savingProfile ? 'hourglass_empty' : 'save' }}</span>
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

        <section v-if="canEdit" class="card connected-accounts-card">
          <h2 class="profile-section-title">Connected Accounts</h2>

          <!-- Battle.net -->
          <div class="connected-account-row">
            <div class="connected-account-info">
              <img :src="battlenetLogo" class="connected-account-logo connected-account-logo-bnet" alt="" aria-hidden="true" />
              <div class="connected-account-label-wrap">
                <span class="connected-account-label">Battle.net</span>
                <span v-if="profile.battletag" class="connected-account-sublabel">{{ profile.battletag }}</span>
              </div>
              <span v-if="!profile.can_edit_battletag" class="connected-account-badge">Connected</span>
            </div>
            <div class="connected-account-actions">
              <p v-if="!profile.can_edit_battletag && !profile.has_password" class="connected-account-warning">
                Set a password before disconnecting, otherwise you will lose access.
              </p>
              <button
                v-if="!profile.can_edit_battletag"
                type="button"
                class="connected-account-btn connected-account-btn-disconnect"
                :disabled="disconnectingBnet || !profile.has_password"
                @click="disconnectBnetAccount"
              >
                {{ disconnectingBnet ? 'Disconnecting...' : 'Disconnect' }}
              </button>
              <button
                v-else
                type="button"
                class="connected-account-btn connected-account-btn-connect"
                :disabled="connectingBnet"
                @click="connectBnetAccount"
              >
                {{ connectingBnet ? 'Connecting...' : 'Connect' }}
              </button>
            </div>
          </div>

          <!-- Discord -->
          <div class="connected-account-row">
            <div class="connected-account-info">
              <svg class="connected-account-logo connected-account-logo-discord" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/></svg>
              <div class="connected-account-label-wrap">
                <span class="connected-account-label">Discord</span>
                <span v-if="profile.discord_username" class="connected-account-sublabel">{{ profile.discord_username }}</span>
              </div>
              <span v-if="profile.has_discord_identity" class="connected-account-badge connected-account-badge-discord">Connected</span>
            </div>
            <div class="connected-account-actions">
              <p v-if="profile.has_discord_identity && !profile.has_password" class="connected-account-warning">
                Set a password before disconnecting, otherwise you will lose access.
              </p>
              <button
                v-if="profile.has_discord_identity"
                type="button"
                class="connected-account-btn connected-account-btn-disconnect"
                :disabled="disconnectingDiscord || !profile.has_password"
                @click="disconnectDiscordAccount"
              >
                {{ disconnectingDiscord ? 'Disconnecting...' : 'Disconnect' }}
              </button>
              <button
                v-else
                type="button"
                class="connected-account-btn connected-account-btn-connect connected-account-btn-connect-discord"
                :disabled="connectingDiscord"
                @click="connectDiscordAccount"
              >
                {{ connectingDiscord ? 'Connecting...' : 'Connect' }}
              </button>
            </div>
          </div>
        </section>
      </div>

      <div class="profile-column profile-column-right">
        <ProfileGamesCard
          :profile="profile"
          :can-edit="canEdit"
          :overwatch-summary-rows="overwatchSummaryRows"
          :overwatch-logo="overwatchLogo"
        >
          <template v-if="canEdit" #overwatch-ranks>
            <template v-if="!editingRanks">
              <div class="rank-tiles-wrap">
                <div class="rank-tiles-header">
                  <button class="rank-edit-btn" type="button" @click="startEdit('ranks')">
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
                  <span class="material-symbols-rounded" aria-hidden="true">{{ savingProfile ? 'hourglass_empty' : 'save' }}</span>
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

    <section v-if="profile" class="profile-events-section">
      <div class="profile-events-header">
        <h2 class="profile-section-title">{{ profile.display_name }}'s last 5 events</h2>
        <InlineArrowLink
          :to="{ name: 'events', query: { owner: profileId } }"
          label="See all"
        />
      </div>
      <p v-if="loadingUserEvents" class="muted">Loading...</p>
      <p v-else-if="userEvents.length === 0" class="muted">No events yet.</p>
      <ul v-else class="profile-events-list">
        <EventListItem
          v-for="event in userEvents"
          :key="event.id"
          :event="event"
          :to="{ name: 'event', params: { id: event.id } }"
        />
      </ul>
    </section>

    <section v-else class="card">
      <h2>Profile not found</h2>
      <p class="muted">The user may not exist.</p>
      <button class="btn-secondary" @click="goToEvents">Back to events</button>
    </section>
  </main>
</template>

<style scoped>
.profile-events-section {
  display: grid;
  gap: 0.6rem;
  margin-top: 0.8rem;
}

.profile-events-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.profile-section-title {
  margin: 0;
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
}

.profile-events-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.4rem;
}

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
  align-items: stretch;
}

.profile-layout :deep(.card) {
  background: transparent;
}

.profile-column {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
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

.oauth-redirect-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.62);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
}

.oauth-redirect-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-lg);
  padding: 2rem 2.5rem;
  font-weight: 600;
  color: var(--ink-1);
}

.oauth-redirect-box p {
  margin: 0;
}

.oauth-redirect-spinner {
  display: block;
  width: 2rem;
  height: 2rem;
  border: 3px solid color-mix(in srgb, var(--brand-1) 25%, transparent);
  border-top-color: var(--brand-1);
  border-radius: 50%;
  animation: oauth-spin 0.7s linear infinite;
}

@keyframes oauth-spin {
  to { transform: rotate(360deg); }
}

.connected-accounts-card {
  display: grid;
  gap: 0.85rem;
  border: none;
}

.connected-account-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  flex-wrap: wrap;
}

.connected-account-row + .connected-account-row {
  padding-top: 0.85rem;
  border-top: 1px solid color-mix(in srgb, var(--line) 60%, transparent);
}

.connected-account-info {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  font-weight: 600;
}

.connected-account-label-wrap {
  display: flex;
  flex-direction: column;
  gap: 0.05rem;
}

.connected-account-sublabel {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--ink-muted);
}

.connected-account-logo {
  width: 1.35rem;
  height: 1.35rem;
  flex-shrink: 0;
}

.connected-account-logo-bnet {
  border-radius: var(--radius-pill);
}

.connected-account-logo-discord {
  color: #5865f2;
}

.connected-account-badge {
  font-size: 0.75rem;
  font-weight: 700;
  color: #148eff;
  background: color-mix(in srgb, #148eff 12%, transparent);
  border-radius: var(--radius-pill);
  padding: 0.15rem 0.55rem;
}

.connected-account-badge-discord {
  color: #5865f2;
  background: color-mix(in srgb, #5865f2 12%, transparent);
}

.connected-account-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.4rem;
}

.connected-account-warning {
  font-size: 0.8rem;
  color: var(--ink-muted);
  max-width: 220px;
  text-align: right;
  margin: 0;
}

.connected-account-btn {
  border: none;
  border-radius: var(--radius-md);
  padding: 0.4rem 0.85rem;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 120ms ease;
}

.connected-account-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.connected-account-btn-connect {
  background: #148eff;
  color: #fff;
}

.connected-account-btn-connect:hover:not(:disabled) {
  background: #1a9aff;
}

.connected-account-btn-connect-discord {
  background: #5865f2;
}

.connected-account-btn-connect-discord:hover:not(:disabled) {
  background: #6470f3;
}

.connected-account-btn-disconnect {
  background: color-mix(in srgb, var(--card) 60%, var(--line));
  color: var(--ink-1);
}

.connected-account-btn-disconnect:hover:not(:disabled) {
  background: color-mix(in srgb, var(--card) 40%, var(--line));
}
</style>
