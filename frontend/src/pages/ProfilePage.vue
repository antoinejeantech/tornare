<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { apiCall } from '../lib/api'
import overwatchLogo from '../assets/branding/overwatch-logo-gold.png'
import { getRankIcon, overwatchRanks } from '../lib/ranks'
import { useAuthStore } from '../stores/auth'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const profile = ref(null)
const loadingProfile = ref(false)
const savingProfile = ref(false)
const error = ref('')
const notice = ref('')
const profileFormTouched = ref(false)
const editingAccount = ref(false)
const editingOverwatch = ref(false)
const editUsername = ref('')
const editDisplayName = ref('')
const editEmail = ref('')
const editBattletag = ref('')
const editRankTank = ref('Unranked')
const editRankDps = ref('Unranked')
const editRankSupport = ref('Unranked')
const editPassword = ref('')
const editPasswordConfirm = ref('')

const profileId = computed(() => String(route.params.id || ''))
const viewerId = computed(() => String(authStore.user?.id || ''))
const canEdit = computed(() => authStore.isAuthenticated && profileId.value && viewerId.value === profileId.value)
const profileInitial = computed(() => {
  const label = String(profile.value?.display_name || profile.value?.username || '').trim()
  return label.length > 0 ? label[0].toUpperCase() : 'A'
})
const battletagStateLabel = computed(() => {
  return profile.value?.battletag ? 'Battletag set' : 'Battletag missing'
})
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
  return canSaveProfile.value && editingOverwatch.value && hasOverwatchChanges.value
})

const overwatchSummaryRows = computed(() => {
  if (!profile.value) {
    return []
  }

  return [
    { role: 'Tank', rank: profile.value.rank_tank || 'Unranked' },
    { role: 'DPS', rank: profile.value.rank_dps || 'Unranked' },
    { role: 'Support', rank: profile.value.rank_support || 'Unranked' },
  ]
})

function markProfileFormTouched() {
  profileFormTouched.value = true
}

function hydrateFormFromProfile(value) {
  if (!value) {
    return
  }

  editDisplayName.value = value.display_name || ''
  editUsername.value = value.username || ''
  editEmail.value = value.email || ''
  editBattletag.value = value.battletag || ''
  editRankTank.value = value.rank_tank || 'Unranked'
  editRankDps.value = value.rank_dps || 'Unranked'
  editRankSupport.value = value.rank_support || 'Unranked'
  editPassword.value = ''
  editPasswordConfirm.value = ''
  profileFormTouched.value = false
}

function startEdit(section) {
  if (section === 'account') {
    editingAccount.value = true
    editingOverwatch.value = false
    return
  }

  if (section === 'overwatch') {
    editingOverwatch.value = true
    editingAccount.value = false
  }
}

function cancelEditSection(section) {
  hydrateFormFromProfile(profile.value)
  if (section === 'account') {
    editingAccount.value = false
  }
  if (section === 'overwatch') {
    editingOverwatch.value = false
  }
}

function setError(message) {
  error.value = message
  notice.value = ''
}

function setNotice(message) {
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
    const response = await apiCall(`/api/users/${profileId.value}`)
    profile.value = response
    hydrateFormFromProfile(response)
    editingAccount.value = false
    editingOverwatch.value = false
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
    const updated = await apiCall(`/api/users/${profileId.value}`, {
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
    authStore.user = {
      ...(authStore.user || {}),
      ...updated,
    }
    editingAccount.value = false
    editingOverwatch.value = false
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
    editingOverwatch.value = false
  }
)

onMounted(async () => {
  await authStore.initialize()
  await loadProfile()
})
</script>

<template>
  <main class="app-shell profile-shell">
    <header class="page-header">
      <h1 class="page-title">Profile</h1>
    </header>

    <p v-if="error" class="status status-error">{{ error }}</p>
    <p v-else-if="notice" class="status status-ok">{{ notice }}</p>

    <section v-if="loadingProfile" class="card">
      <p>Loading profile...</p>
    </section>

    <section v-else-if="profile" class="profile-layout">
      <article class="card profile-section account-section">
        <div class="profile-cover" aria-hidden="true"></div>
        <div class="profile-header">
          <div class="profile-identity">
            <span class="profile-avatar" aria-hidden="true">{{ profileInitial }}</span>
            <div class="profile-headline">
              <h2>{{ profile.display_name }}</h2>
              <p class="muted">{{ profile.username }}</p>
              <p v-if="profile.battletag" class="muted">{{ profile.battletag }}</p>
              <div class="profile-stat-chips">
                <span class="meta-chip">Main game: Overwatch</span>
                <span class="meta-chip">Role: {{ profile.role || 'user' }}</span>
                <span class="meta-chip">Roles tracked: 3</span>
                <span class="meta-chip">{{ battletagStateLabel }}</span>
              </div>
            </div>
          </div>
          <button
            v-if="canEdit && !editingAccount"
            type="button"
            class="btn-secondary section-edit-btn"
            @click="startEdit('account')"
          >
            Edit
          </button>
        </div>

        <form v-if="editingAccount" class="profile-form animated-panel" @submit.prevent="saveProfile">
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
          <div class="section-actions">
            <button type="submit" class="btn-primary" :disabled="!canSaveAccountSection">
              {{ savingProfile ? 'Saving...' : 'Save account' }}
            </button>
            <button type="button" class="btn-secondary" :disabled="savingProfile" @click="cancelEditSection('account')">
              Cancel
            </button>
          </div>
        </form>

        <div v-else class="profile-summary-grid animated-panel">
          <p><strong>Username:</strong> {{ profile.username }}</p>
          <p><strong>Display name:</strong> {{ profile.display_name }}</p>
          <p><strong>Role:</strong> {{ profile.role || 'user' }}</p>
          <p v-if="canEdit"><strong>Email:</strong> {{ profile.email }}</p>
        </div>
      </article>

      <article class="card profile-section">
        <div class="section-header">
          <h3 class="section-title">
            <span class="material-symbols-rounded" aria-hidden="true">sports_esports</span>
            <span>Games</span>
          </h3>
          <button
            v-if="canEdit && !editingOverwatch"
            type="button"
            class="btn-secondary section-edit-btn"
            @click="startEdit('overwatch')"
          >
            Edit
          </button>
        </div>

        <section class="game-card">
          <h4 class="game-title">
            <img class="game-logo" :src="overwatchLogo" alt="Overwatch" />
            <span>Overwatch</span>
          </h4>

          <form v-if="editingOverwatch" class="profile-form animated-panel" @submit.prevent="saveProfile">
            <label>
              Battletag
              <input
                v-model="editBattletag"
                placeholder="Player#1234"
                disabled
              />
            </label>
            <p class="muted profile-note">
              Battletag will be editable via Battle.net OAuth once connected.
            </p>
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
            <div class="section-actions">
              <button type="submit" class="btn-primary" :disabled="!canSaveOverwatchSection">
                {{ savingProfile ? 'Saving...' : 'Save Overwatch' }}
              </button>
              <button type="button" class="btn-secondary" :disabled="savingProfile" @click="cancelEditSection('overwatch')">
                Cancel
              </button>
            </div>
          </form>

          <div v-else class="profile-summary-grid animated-panel">
            <p v-if="profile.battletag"><strong>Battletag:</strong> {{ profile.battletag }}</p>
            <div v-else class="battletag-empty">
              <span class="muted">No battletag configured yet. Connect Battle.net (OAuth) to sync it.</span>
            </div>
            <div class="summary-rank-list">
              <article v-for="entry in overwatchSummaryRows" :key="entry.role" class="summary-rank-row">
                <strong class="summary-rank-role">{{ entry.role }}</strong>
                <span class="summary-rank-value">
                  <img class="summary-rank-icon" :src="getRankIcon(entry.rank)" :alt="`${entry.rank} rank`" />
                  <span>{{ entry.rank }}</span>
                </span>
              </article>
            </div>
          </div>
        </section>
      </article>
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
  max-width: 1820px;
  width: min(96vw, 1820px);
}

.profile-card {
  display: grid;
  gap: 0.7rem;
}

.profile-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 0.8rem;
  align-items: stretch;
}

.profile-section {
  padding: 1rem 1.15rem;
  min-height: 420px;
}

.account-section {
  position: relative;
  overflow: hidden;
}

.profile-cover {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 130px;
  background:
    radial-gradient(640px 130px at 4% 0%, rgba(95, 164, 255, 0.24), transparent 70%),
    radial-gradient(520px 130px at 100% 0%, rgba(45, 123, 231, 0.2), transparent 72%);
  pointer-events: none;
}

.profile-form {
  display: grid;
  gap: 0.55rem;
  max-width: 960px;
}

.profile-form label {
  display: grid;
  gap: 0.25rem;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.55rem;
  margin-top: 0.7rem;
}

.section-title {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}

.section-header h3,
.game-card h4 {
  margin: 0;
}

.section-edit-btn {
  padding: 0.3rem 0.72rem;
}

.section-actions {
  display: flex;
  gap: 0.45rem;
  align-items: center;
}

.profile-summary-grid {
  display: grid;
  gap: 0.38rem;
}

.profile-summary-grid p {
  margin: 0;
}

.game-card {
  border: 1px solid color-mix(in srgb, var(--line) 88%, var(--brand-1) 12%);
  border-radius: 12px;
  background: color-mix(in srgb, var(--card) 94%, #f1f6ff 6%);
  padding: 0.75rem;
  display: grid;
  gap: 0.75rem;
  min-height: 320px;
  grid-auto-rows: max-content;
}

.game-title {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}

.game-logo {
  width: 1.25rem;
  height: 1.25rem;
  object-fit: contain;
}

.summary-rank-list {
  display: grid;
  gap: 0.45rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.summary-rank-row {
  display: grid;
  gap: 0.42rem;
  margin: 0;
  padding: 0.62rem 0.68rem;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-2) 14%);
  background: color-mix(in srgb, var(--card) 88%, #19253a 12%);
  box-shadow: 0 6px 14px rgba(19, 53, 116, 0.12);
}

.summary-rank-role {
  min-width: 0;
  font-size: 0.84rem;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--ink-2);
}

.summary-rank-value {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-weight: 760;
}

.summary-rank-icon {
  width: 22px;
  height: 22px;
  object-fit: contain;
}

.profile-note {
  margin: -0.2rem 0 0.1rem;
}

.profile-ranks-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.7rem;
}

.profile-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.85rem;
  margin-bottom: 1rem;
  position: relative;
  z-index: 1;
}

.profile-identity {
  display: inline-flex;
  align-items: center;
  gap: 1.1rem;
  min-width: 0;
}

.profile-headline {
  display: grid;
  gap: 0.3rem;
}

.profile-stat-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.meta-chip {
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--brand-1) 35%, var(--line) 65%);
  background: color-mix(in srgb, var(--accent) 22%, var(--meta-bg) 78%);
  color: var(--meta-ink);
  padding: 0.18rem 0.55rem;
  font-size: 0.74rem;
  font-family: "Avenir Next", "Segoe UI", "Helvetica Neue", sans-serif;
  font-weight: 700;
  text-transform: uppercase;
}

.battletag-empty {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.45rem;
}

.battletag-cta {
  padding: 0.28rem 0.62rem;
}

.animated-panel {
  animation: rise-in 220ms ease-out;
}

.profile-headline h2 {
  margin: 0;
}

.profile-headline p {
  margin: 0;
}

.profile-avatar {
  width: 2.4rem;
  height: 2.4rem;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
  font-weight: 800;
  color: #fff;
  background: linear-gradient(130deg, var(--brand-2), var(--brand-1));
  border: 1px solid color-mix(in srgb, var(--brand-2) 66%, var(--brand-1) 34%);
  box-shadow: 0 4px 10px rgba(78, 52, 7, 0.26);
}

@media (max-width: 980px) {
  .profile-layout {
    grid-template-columns: 1fr;
  }

  .profile-ranks-grid {
    grid-template-columns: 1fr;
  }

  .summary-rank-list {
    grid-template-columns: 1fr;
  }

  .section-actions {
    flex-wrap: wrap;
  }
}
</style>
