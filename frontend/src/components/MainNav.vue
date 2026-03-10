<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { RouterLink } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import tornareLogo from '../assets/branding/tornare-logo.svg'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const mobileMenuOpen = ref(false)
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

function applyTheme(mode) {
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
})

onMounted(() => {
  authStore.initialize()

  if (typeof window !== 'undefined') {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
    if (stored === 'light' || stored === 'dark') {
      applyTheme(stored)
      return
    }

    const prefersLight = window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches
    applyTheme(prefersLight ? 'light' : 'dark')
    return
  }

  applyTheme('dark')
})
</script>

<template>
  <nav class="top-nav">
    <div class="top-nav-inner">
      <RouterLink class="brand-link" to="/" aria-label="Tornare">
        <img class="brand-logo" :src="tornareLogo" alt="" aria-hidden="true" />
        <span class="brand-wordmark">ornare</span>
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
        <button v-if="!authStore.isAuthenticated" class="top-nav-link top-nav-theme-toggle" type="button" :title="themeLabel()" @click="toggleTheme">
          <span class="material-symbols-rounded" aria-hidden="true">{{ themeIcon() }}</span>
          <span>{{ themeLabel() }}</span>
        </button>
        <div class="top-nav-fake-search" aria-hidden="true">
          <span class="material-symbols-rounded" aria-hidden="true">search</span>
          <span>Search</span>
        </div>
        <RouterLink v-if="!authStore.isAuthenticated" class="top-nav-link" :to="loginRoute" @click="closeMobileMenu">
          <span class="material-symbols-rounded" aria-hidden="true">login</span>
          <span>Login</span>
        </RouterLink>
        <div v-else class="top-nav-user-controls desktop-only">
          <div class="top-nav-user-menu" tabindex="0">
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
          <button class="top-nav-link top-nav-theme-toggle top-nav-theme-toggle-compact" type="button" :title="themeLabel()" @click="toggleTheme">
            <span class="material-symbols-rounded" aria-hidden="true">{{ themeIcon() }}</span>
            <span class="sr-only">{{ themeLabel() }}</span>
          </button>
        </div>

        <div v-if="authStore.isAuthenticated" class="top-nav-mobile-user mobile-only">
          <RouterLink class="top-nav-link" :to="profileRoute" @click="closeMobileMenu">
            <span class="material-symbols-rounded" aria-hidden="true">person</span>
            <span>Profile</span>
          </RouterLink>
          <button class="top-nav-link top-nav-theme-toggle" type="button" :title="themeLabel()" @click="toggleTheme">
            <span class="material-symbols-rounded" aria-hidden="true">{{ themeIcon() }}</span>
            <span>{{ themeLabel() }}</span>
          </button>
          <button class="top-nav-link top-nav-mobile-logout" type="button" @click="logout">
            <span class="material-symbols-rounded" aria-hidden="true">logout</span>
            <span>Logout</span>
          </button>
        </div>
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
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--card) 82%, var(--nav-mix) 18%), color-mix(in srgb, var(--card) 90%, var(--nav-mix) 10%));
  backdrop-filter: blur(10px);
  box-shadow: 0 8px 22px rgba(14, 32, 72, 0.15);
}

.top-nav-inner {
  max-width: 1820px;
  width: min(96vw, 1820px);
  margin: 0 auto;
  padding: 0.8rem 1rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
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
  margin-left: auto;
}

.top-nav-mobile-toggle {
  display: none;
  border: 1px solid color-mix(in srgb, var(--brand-2) 44%, var(--line) 56%);
  background: color-mix(in srgb, var(--card) 92%, var(--brand-2) 8%);
  color: var(--ink-1);
  border-radius: 10px;
}

.top-nav-mobile-toggle .material-symbols-rounded {
  font-size: 1.1rem;
}

.top-nav-link {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  text-decoration: none;
  padding: 0.38rem 0.72rem;
  border-radius: 999px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-muted);
  font-weight: 620;
  letter-spacing: 0.01em;
  transition: box-shadow 0.16s ease, background 0.16s ease, border-color 0.16s ease, transform 0.12s ease;
}

.top-nav-theme-toggle {
  cursor: pointer;
}

.top-nav-theme-toggle-compact {
  min-width: 2.25rem;
  padding: 0.38rem 0.52rem;
  justify-content: center;
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
  transform: none;
}

.top-nav-link:hover .material-symbols-rounded {
  color: color-mix(in srgb, var(--ink-1) 92%, #fff 8%);
}

.top-nav-link:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--brand-2) 50%, white 50%);
  outline-offset: 1px;
}

.top-nav-link.router-link-active {
  color: #fff;
  border-color: color-mix(in srgb, var(--brand-2) 70%, var(--brand-1) 30%);
  background: linear-gradient(130deg, var(--brand-2), var(--brand-1));
  box-shadow: 0 8px 18px rgba(78, 52, 7, 0.3);
}

.top-nav-link.router-link-active .material-symbols-rounded {
  color: currentColor;
}

.top-nav-fake-search {
  min-width: 140px;
  display: inline-flex;
  align-items: center;
  gap: 0.28rem;
  padding: 0.34rem 0.62rem;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, #5b6f93 46%, var(--line) 54%);
  background: linear-gradient(180deg, #0b101b 0%, #0e1523 100%);
  color: var(--ink-muted);
  font-size: 0.78rem;
  font-weight: 650;
  letter-spacing: 0.01em;
  box-shadow: inset 0 1px 0 rgba(188, 210, 255, 0.08);
}

.top-nav-fake-search .material-symbols-rounded {
  font-size: 0.92rem;
  color: color-mix(in srgb, var(--ink-muted) 88%, var(--ink-1) 12%);
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

.top-nav-user-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  min-width: 140px;
  padding: 0.32rem;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--brand-2) 34%, var(--line) 66%);
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--card) 90%, #18253a 10%) 0%, color-mix(in srgb, var(--card) 96%, #101828 4%) 100%);
  box-shadow: 0 12px 26px rgba(3, 8, 18, 0.42);
  opacity: 0;
  transform: translateY(-4px);
  pointer-events: none;
  transition: opacity 0.14s ease, transform 0.14s ease;
}

.top-nav-user-menu:hover .top-nav-user-dropdown,
.top-nav-user-menu:focus-within .top-nav-user-dropdown {
  opacity: 1;
  transform: translateY(0);
  pointer-events: auto;
}

.top-nav-user-action {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  width: 100%;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--brand-1);
  text-align: left;
  text-decoration: none;
  padding: 0.46rem 0.56rem;
  font-weight: 700;
  cursor: pointer;
}

.top-nav-user-action + .top-nav-user-action {
  border-top: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
}

.top-nav-user-action .material-symbols-rounded {
  font-size: 1rem;
}

.top-nav-user-action:hover {
  background: color-mix(in srgb, var(--brand-2) 16%, var(--card) 84%);
  color: color-mix(in srgb, var(--brand-1) 86%, #fff 14%);
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
    border-radius: 12px;
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
    border-radius: 10px;
    padding: 0.52rem 0.62rem;
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

  .top-nav-fake-search {
    min-width: 104px;
    padding-inline: 0.52rem;
  }
}
</style>
