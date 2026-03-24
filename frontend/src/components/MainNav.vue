<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { RouterLink } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { apiCall } from '../lib/api'
import { useDebounce } from '../composables/useDebounce'
import { useRequestSequence } from '../composables/useRequestSequence'
import tornareLogo from '../assets/branding/tornare-logo-pulse.svg'

interface UserSearchResult {
  id: string
  username: string
  display_name: string
}

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const mobileMenuOpen = ref(false)
const notificationsOpen = ref(false)
const searchQuery = ref('')
const searchResults = ref<UserSearchResult[]>([])
const searchOpen = ref(false)
const { debounced: debouncedSearch, cancel: cancelSearch } = useDebounce(300)
const { next: nextSearchId, isCurrent: isCurrentSearch, invalidate: invalidateSearch } = useRequestSequence()

function onSearchInput() {
  const q = searchQuery.value.trim()
  if (!q) {
    invalidateSearch()
    searchResults.value = []
    searchOpen.value = false
    return
  }
  debouncedSearch(() => fetchSearchResults(q))
}

async function fetchSearchResults(q: string) {
  const requestId = nextSearchId()

  try {
    const res = await apiCall<UserSearchResult[]>(`/api/users?search=${encodeURIComponent(q)}`)

    if (!isCurrentSearch(requestId) || q !== searchQuery.value.trim()) {
      return
    }

    searchResults.value = res
    searchOpen.value = res.length > 0
  } catch {
    if (!isCurrentSearch(requestId) || q !== searchQuery.value.trim()) {
      return
    }

    searchResults.value = []
    searchOpen.value = false
  }
}

function goToProfile(id: string) {
  router.push({ name: 'profile', params: { id } })
  clearSearch()
}

function clearSearch() {
  invalidateSearch()
  cancelSearch()
  searchQuery.value = ''
  searchResults.value = []
  searchOpen.value = false
}
const themeMode = ref('dark')
const THEME_STORAGE_KEY = 'tornare_theme'

const loginRoute = computed(() => {
  const redirect = route.name === 'auth' ? '/events' : route.fullPath
  return { name: 'auth', query: { redirect } }
})
const authLabel = computed(() => authStore.user?.display_name || 'Account')
const profileRoute = computed(() => {
  const id = String(authStore.user?.id || '').trim()
  if (!id) {
    return { name: 'events' }
  }

  return { name: 'profile', params: { id } }
})

async function logout() {
  mobileMenuOpen.value = false
  await authStore.logout()
  router.push({ name: 'home' })
}

function toggleMobileMenu() {
  mobileMenuOpen.value = !mobileMenuOpen.value
}

function closeMobileMenu() {
  mobileMenuOpen.value = false
}

function toggleNotifications() {
  notificationsOpen.value = !notificationsOpen.value
}

function closeNotifications() {
  notificationsOpen.value = false
}

function handleDocumentClick(event: MouseEvent) {
  const target = event?.target
  if (!(target instanceof Element)) {
    return
  }

  if (!target.closest('.top-nav-notification')) {
    closeNotifications()
  }
  if (!target.closest('.top-nav-search')) {
    searchOpen.value = false
  }
}

function applyTheme(mode: string) {
  if (typeof document === 'undefined') {
    return
  }

  themeMode.value = mode === 'light' ? 'light' : 'dark'
  document.body.classList.toggle('theme-light', themeMode.value === 'light')
}

function toggleTheme() {
  const next = themeMode.value === 'light' ? 'dark' : 'light'
  applyTheme(next)

  if (typeof window !== 'undefined') {
    window.localStorage.setItem(THEME_STORAGE_KEY, next)
  }
}

function themeIcon() {
  return themeMode.value === 'light' ? 'dark_mode' : 'light_mode'
}

function themeLabel() {
  return themeMode.value === 'light' ? 'Dark mode' : 'Light mode'
}

watch(() => route.fullPath, () => {
  closeMobileMenu()
  closeNotifications()
})

onMounted(() => {
  if (typeof window !== 'undefined') {
    // Never auto-switch to browser light preference.
    applyTheme('dark')
    window.localStorage.setItem(THEME_STORAGE_KEY, 'dark')
    return
  }

  applyTheme('dark')
})

onMounted(() => {
  if (typeof document !== 'undefined') {
    document.addEventListener('click', handleDocumentClick)
  }
})

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') {
    document.removeEventListener('click', handleDocumentClick)
  }
})
</script>

<template>
  <nav class="top-nav">
    <div class="top-nav-inner">
      <RouterLink class="brand-link" to="/" aria-label="Tornare">
        <img class="brand-logo" :src="tornareLogo" alt="" aria-hidden="true" />
        <span class="brand-wordmark">tornare</span>
      </RouterLink>
      <button
        class="top-nav-mobile-toggle icon-btn"
        type="button"
        :aria-expanded="mobileMenuOpen ? 'true' : 'false'"
        aria-controls="top-nav-mobile-menu"
        :title="mobileMenuOpen ? 'Close navigation menu' : 'Open navigation menu'"
        @click="toggleMobileMenu"
      >
        <span class="material-symbols-rounded" aria-hidden="true">{{ mobileMenuOpen ? 'close' : 'menu' }}</span>
        <span class="sr-only">{{ mobileMenuOpen ? 'Close navigation menu' : 'Open navigation menu' }}</span>
      </button>
      <div id="top-nav-mobile-menu" class="top-nav-links" :class="{ 'menu-open': mobileMenuOpen }">
        <RouterLink class="top-nav-link" to="/" @click="closeMobileMenu">
          <span class="material-symbols-rounded" aria-hidden="true">home</span>
          <span>Home</span>
        </RouterLink>
        <RouterLink class="top-nav-link" to="/events" @click="closeMobileMenu">
          <span class="material-symbols-rounded" aria-hidden="true">event</span>
          <span>Events</span>
        </RouterLink>
        <RouterLink class="top-nav-link" to="/about" @click="closeMobileMenu">
          <span class="material-symbols-rounded" aria-hidden="true">info</span>
          <span>About</span>
        </RouterLink>
        <RouterLink class="top-nav-link" to="/news" @click="closeMobileMenu">
          <span class="material-symbols-rounded" aria-hidden="true">article</span>
          <span>News</span>
        </RouterLink>
        <div class="top-nav-search" @keydown.escape="clearSearch">
          <span class="material-symbols-rounded top-nav-search-icon" aria-hidden="true">search</span>
          <input
            id="main-nav-user-search"
            v-model="searchQuery"
            class="top-nav-search-input"
            type="search"
            placeholder="Search users..."
            aria-label="Search users"
            autocomplete="off"
            @input="onSearchInput"
          />
          <div v-if="searchOpen && searchResults.length" class="top-nav-search-dropdown">
            <button
              v-for="user in searchResults"
              :key="user.id"
              class="top-nav-search-result"
              type="button"
              @click="goToProfile(user.id)"
            >
              <span class="result-display-name">{{ user.display_name }}</span>
              <span class="result-username">@{{ user.username }}</span>
            </button>
          </div>
        </div>
        <RouterLink v-if="!authStore.isAuthenticated" class="top-nav-link" :to="loginRoute" @click="closeMobileMenu">
          <span class="material-symbols-rounded" aria-hidden="true">login</span>
          <span>Login</span>
        </RouterLink>
        <div v-else class="top-nav-user-controls desktop-only">
          <div class="top-nav-notification">
            <button
              class="top-nav-link top-nav-notification-btn"
              type="button"
              title="Notifications"
              aria-controls="top-nav-notifications-panel"
              :aria-expanded="notificationsOpen ? 'true' : 'false'"
              @click="toggleNotifications"
            >
              <span class="material-symbols-rounded" aria-hidden="true">notifications</span>
              <span class="sr-only">Notifications</span>
            </button>
            <div v-if="notificationsOpen" id="top-nav-notifications-panel" class="top-nav-notifications-panel" role="dialog" aria-label="Notifications panel">
              <p class="top-nav-notifications-empty">No notifications right now.</p>
            </div>
          </div>
          <div class="top-nav-user-menu">
            <button class="top-nav-user-trigger" type="button">
              <span>{{ authLabel }}</span>
              <span class="material-symbols-rounded" aria-hidden="true">expand_more</span>
            </button>
            <div class="top-nav-user-dropdown" role="menu" aria-label="User menu">
              <RouterLink class="top-nav-user-action" :to="profileRoute">
                <span class="material-symbols-rounded" aria-hidden="true">person</span>
                <span>Profile</span>
              </RouterLink>
              <button class="top-nav-user-action" type="button" @click="logout">
                <span class="material-symbols-rounded" aria-hidden="true">logout</span>
                <span>Logout</span>
              </button>
            </div>
          </div>
        </div>

        <div v-if="authStore.isAuthenticated" class="top-nav-mobile-user mobile-only">
          <RouterLink class="top-nav-link" :to="profileRoute" @click="closeMobileMenu">
            <span class="material-symbols-rounded" aria-hidden="true">person</span>
            <span>Profile</span>
          </RouterLink>
          <button class="top-nav-link top-nav-mobile-logout" type="button" @click="logout">
            <span class="material-symbols-rounded" aria-hidden="true">logout</span>
            <span>Logout</span>
          </button>
        </div>

        <button class="top-nav-link top-nav-theme-toggle top-nav-theme-toggle-compact" type="button" :title="themeLabel()" :disabled="true" aria-disabled="true" @click="toggleTheme">
          <span class="material-symbols-rounded" aria-hidden="true">{{ themeIcon() }}</span>
          <span class="sr-only">{{ themeLabel() }}</span>
        </button>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.top-nav {
  position: sticky;
  top: 0;
  z-index: 50;
  border-bottom: 1px solid color-mix(in srgb, var(--brand-1) 20%, var(--line) 80%);
  background: var(--bg-0);
  backdrop-filter: blur(var(--blur-md));
  box-shadow: none;
}

.top-nav-inner {
  max-width: 1820px;
  width: min(96vw, 1820px);
  margin: 0 auto;
  padding: 0.8rem 1rem;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.8rem;
}

.brand-link {
  display: inline-flex;
  align-items: center;
  gap: 0.18rem;
  text-decoration: none;
  color: var(--brand-1);
  font-size: 0.9rem;
  font-weight: 500;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  text-shadow: 1px 1px 0 rgba(0, 0, 0, 0.22);
  line-height: 1;
}

.brand-link:hover {
  color: color-mix(in srgb, var(--brand-1) 86%, #fff 14%);
}

.brand-logo {
  display: block;
  width: 2rem;
  height: 2rem;
  flex: 0 0 auto;
  transform: translateY(-0.01em);
  object-fit: contain;
  filter: drop-shadow(0 2px 6px rgba(154, 114, 50, 0.28));
}

.brand-wordmark {
  display: inline-flex;
  align-items: center;
  line-height: 1;
  transform: translateY(0.1em);
}

.top-nav-links {
  display: flex;
  align-items: center;
  gap: 0.42rem;
  flex: 1;
}

.top-nav-mobile-toggle {
  display: none;
  border: 1px solid color-mix(in srgb, var(--brand-2) 44%, var(--line) 56%);
  background: color-mix(in srgb, var(--card) 92%, var(--brand-2) 8%);
  color: var(--ink-1);
  border-radius: var(--radius-md);
}

.top-nav-mobile-toggle .material-symbols-rounded {
  font-size: 1.1rem;
}

.top-nav-link {
  display: inline-flex;
  position: relative;
  align-items: center;
  gap: 0.3rem;
  text-decoration: none;
  padding: 0.38rem 0.72rem;
  border-radius: var(--radius-pill);
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-2);
  font-weight: 620;
  letter-spacing: 0.01em;
  transition: box-shadow 0.16s ease, background 0.16s ease, border-color 0.16s ease, transform 0.12s ease;
}

.top-nav-link::after {
  content: '';
  position: absolute;
  left: 0.72rem;
  right: 0.72rem;
  bottom: 0.16rem;
  height: 2px;
  border-radius: var(--radius-pill);
  background: var(--accent);
  transform: scaleX(0);
  transform-origin: center;
  transition: transform 0.16s ease;
}

.top-nav-theme-toggle {
  cursor: pointer;
}

.top-nav-theme-toggle:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.top-nav-theme-toggle-compact {
  min-width: 2.25rem;
  padding: 0.38rem 0.52rem;
  justify-content: center;
}

.top-nav-theme-toggle-compact span:not(.sr-only) {
  display: inline-flex;
}

.top-nav-theme-toggle-compact > :not(.material-symbols-rounded):not(.sr-only) {
  display: none;
}

.top-nav-theme-toggle-compact .material-symbols-rounded {
  margin: 0;
}

.top-nav-link .material-symbols-rounded {
  font-size: 1rem;
  color: color-mix(in srgb, var(--ink-muted) 88%, var(--ink-1) 12%);
}

.top-nav-link:hover {
  color: var(--ink-1);
}

.top-nav-link:hover .material-symbols-rounded {
  color: var(--ink-1);
}

.top-nav-link:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--brand-2) 50%, white 50%);
  outline-offset: 1px;
}

.top-nav-link.router-link-active {
  color: var(--accent);
  border-color: transparent;
  background: transparent;
  box-shadow: none;
}

.top-nav-link.router-link-active .material-symbols-rounded {
  color: currentColor;
}

.top-nav-link.router-link-active::after,
.top-nav-link.router-link-exact-active::after {
  transform: scaleX(1);
}

.top-nav-search {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.04rem 0.72rem;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--line) 40%, #0a0d12 60%);
  background: color-mix(in srgb, var(--bg-0) 60%, var(--bg-1, #1a1d24) 40%);
  color: var(--ink-2);
  font-weight: 620;
  letter-spacing: 0.01em;
  transition: border-color 0.14s ease, color 0.14s ease;
  margin-left: auto;
}

.top-nav-search:focus-within {
  border-color: color-mix(in srgb, var(--brand-1) 50%, var(--line) 50%);
  color: var(--ink-1);
}

.top-nav-search-icon {
  font-size: 1rem;
  color: color-mix(in srgb, var(--ink-muted) 88%, var(--ink-1) 12%);
  flex-shrink: 0;
}

.top-nav-search:focus-within .top-nav-search-icon {
  color: var(--ink-1);
}

.top-nav-search-input {
  background: transparent;
  border: none;
  outline: none;
  padding-block: 0.15rem;
  color: var(--ink-1);
  font-size: 0.8rem;
  font-weight: 620;
  letter-spacing: 0.01em;
  width: 148px;
  min-width: 0;
}

.top-nav-search-input::placeholder {
  color: var(--ink-muted);
  font-weight: 620;
}

.top-nav-search-input::-webkit-search-cancel-button {
  display: none;
}

.top-nav-search-dropdown {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 220px;
  background: var(--bg-1, #1a1d24);
  border: 1px solid var(--line);
  border-radius: var(--radius-sm);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.32);
  overflow: hidden;
  z-index: 200;
}

.top-nav-search-result {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.1rem;
  width: 100%;
  padding: 0.52rem 0.72rem;
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--line);
  cursor: pointer;
  text-align: left;
  transition: background 0.1s ease;
}

.top-nav-search-result:last-child {
  border-bottom: none;
}

.top-nav-search-result:hover {
  background: color-mix(in srgb, var(--brand-1) 10%, transparent 90%);
}

.result-display-name {
  font-size: 0.84rem;
  font-weight: 700;
  color: var(--ink-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 200px;
}

.result-username {
  font-size: 0.72rem;
  color: var(--ink-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 200px;
}

.top-nav-user-menu {
  position: relative;
  display: inline-flex;
  align-items: center;
}

.top-nav-user-controls {
  display: inline-flex;
  align-items: center;
  gap: 0.34rem;
}

.mobile-only {
  display: none;
}

.top-nav-mobile-user {
  display: none;
  width: 100%;
  gap: 0.38rem;
}

.top-nav-mobile-logout {
  justify-content: flex-start;
  width: 100%;
  cursor: pointer;
}

.top-nav-user-trigger {
  border-radius: 9px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-muted);
  padding: 0.34rem 0.5rem;
  display: inline-flex;
  align-items: center;
  gap: 0.24rem;
  font-size: 0.95rem;
  font-weight: 620;
  font-family: "Avenir Next", "Segoe UI", "Helvetica Neue", sans-serif;
}

.top-nav-user-trigger:hover {
  color: var(--ink-1);
}

.top-nav-user-trigger .material-symbols-rounded {
  font-size: 1rem;
}

.top-nav-notification-btn {
  color: var(--ink-muted);
}

.top-nav-notification-btn .material-symbols-rounded {
  color: currentColor;
}

.top-nav-notification {
  position: relative;
  margin-right: 0.22rem;
  padding-right: 0.56rem;
  border-right: 1px solid color-mix(in srgb, var(--line-strong) 78%, var(--line) 22%);
}

.top-nav-notifications-panel {
  position: absolute;
  top: calc(100% + var(--space-1));
  right: 0;
  width: min(280px, 70vw);
  padding: var(--space-2);
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-1) 16%);
  background: var(--card);
  box-shadow: 0 12px 26px rgba(3, 8, 18, 0.42);
  z-index: 70;
}

.top-nav-notifications-empty {
  margin: 0;
  color: var(--ink-muted);
  font-size: var(--text-sm);
}

.top-nav-user-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  min-width: 140px;
  padding: 0.32rem;
  border-radius: var(--radius-md);
  border: 1px solid var(--line-strong);
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--card) 90%, #18253a 10%) 0%, color-mix(in srgb, var(--card) 96%, #101828 4%) 100%);
  box-shadow: 0 12px 26px rgba(3, 8, 18, 0.42);
  opacity: 0;
  transform: translateY(-4px);
  pointer-events: none;
  transition: opacity 0.14s ease, transform 0.14s ease;
}

.top-nav-user-menu:hover .top-nav-user-dropdown,
.top-nav-user-trigger:focus-visible ~ .top-nav-user-dropdown,
.top-nav-user-menu:focus-within .top-nav-user-dropdown {
  opacity: 1;
  transform: translateY(0);
  pointer-events: auto;
}

.top-nav-user-action {
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.4rem;
  width: 100%;
  border-radius: var(--radius-sm);
  border: none;
  background: transparent;
  color: var(--ink-2);
  text-align: left;
  text-decoration: none;
  padding: 0.46rem 0.56rem;
  font-weight: 700;
  cursor: pointer;
}

.top-nav-user-action + .top-nav-user-action {
  border-top: 1px solid var(--line);
}

.top-nav-user-action .material-symbols-rounded {
  font-size: 1rem;
}

.top-nav-user-action:hover {
  background: color-mix(in srgb, var(--card-soft) 82%, var(--bg-1) 18%);
  color: var(--ink-1);
}

.top-nav-current {
  font-size: 0.83rem;
  color: var(--ink-2);
  font-family: "Space Mono", ui-monospace, monospace;
}

@media (max-width: 900px) {
  .top-nav-inner {
    position: relative;
  }

  .top-nav-mobile-toggle {
    display: inline-flex;
    margin-left: auto;
  }

  .top-nav-links {
    position: absolute;
    top: calc(100% + 0.5rem);
    right: 1rem;
    left: 1rem;
    z-index: 60;
    display: none;
    margin-left: 0;
    padding: 0.55rem;
    border-radius: var(--radius-md);
    border: 1px solid color-mix(in srgb, var(--brand-2) 36%, var(--line) 64%);
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--card) 92%, #18253a 8%) 0%, color-mix(in srgb, var(--card) 96%, #101828 4%) 100%);
    box-shadow: 0 14px 28px rgba(3, 8, 18, 0.46);
    gap: 0.38rem;
    align-items: stretch;
  }

  .top-nav-links.menu-open {
    display: grid;
  }

  .top-nav-link {
    width: 100%;
    justify-content: flex-start;
    border-radius: var(--radius-md);
    padding: 0.52rem 0.62rem;
  }

  .top-nav-theme-toggle-compact {
    width: auto;
    justify-content: center;
    justify-self: end;
    padding: 0.42rem 0.52rem;
  }

  .desktop-only {
    display: none;
  }

  .mobile-only {
    display: block;
  }

  .top-nav-mobile-user {
    display: grid;
  }

  .top-nav-current {
    display: none;
  }

  .top-nav-search {
    padding-inline: 0.52rem;
    margin-left: 0;
  }
  .top-nav-search-input {
    width: 80px;
  }
  .top-nav-search-dropdown {
    min-width: 240px;
  }
}
</style>
