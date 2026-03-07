<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { RouterLink } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const storedTheme = typeof window !== 'undefined' ? window.localStorage.getItem('theme') : null
const initialTheme = storedTheme === 'light' ? 'light' : 'dark'
const theme = ref(initialTheme)

if (typeof document !== 'undefined') {
  document.documentElement.setAttribute('data-theme', initialTheme)
}

function toggleTheme() {
  const nextTheme = theme.value === 'dark' ? 'light' : 'dark'
  theme.value = nextTheme

  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', nextTheme)
  }

  if (typeof window !== 'undefined') {
    window.localStorage.setItem('theme', nextTheme)
  }
}

const themeToggleLabel = computed(() => (theme.value === 'dark' ? 'Light' : 'Dark'))
const themeIcon = computed(() => (theme.value === 'dark' ? 'light_mode' : 'dark_mode'))
const loginRoute = computed(() => {
  const redirect = route.name === 'auth' ? '/events' : route.fullPath
  return { name: 'auth', query: { redirect } }
})
const authLabel = computed(() => authStore.user?.display_name || 'Account')
const authInitial = computed(() => {
  const label = authLabel.value.trim()
  return label.length > 0 ? label[0].toUpperCase() : 'A'
})

async function logout() {
  await authStore.logout()
  router.push({ name: 'home' })
}

onMounted(() => {
  authStore.initialize()
})
</script>

<template>
  <nav class="top-nav">
    <div class="top-nav-inner">
      <RouterLink class="brand-link" to="/">Tornare</RouterLink>
      <div class="top-nav-links">
        <RouterLink class="top-nav-link" to="/">Home</RouterLink>
        <RouterLink class="top-nav-link" to="/events">Events</RouterLink>
        <RouterLink class="top-nav-link" to="/about">About</RouterLink>
        <RouterLink class="top-nav-link" to="/news">News</RouterLink>
        <RouterLink v-if="!authStore.isAuthenticated" class="top-nav-link" :to="loginRoute">Login</RouterLink>
        <div v-else class="top-nav-user-menu" tabindex="0">
          <button class="top-nav-user-trigger" type="button">
            <span class="top-nav-user-avatar" aria-hidden="true">{{ authInitial }}</span>
            <span>{{ authLabel }}</span>
            <span class="material-symbols-rounded" aria-hidden="true">expand_more</span>
          </button>
          <div class="top-nav-user-dropdown" role="menu" aria-label="User menu">
            <RouterLink class="top-nav-user-action" to="/my-events">My Events</RouterLink>
            <button class="top-nav-user-action" type="button" @click="logout">Logout</button>
          </div>
        </div>
        <button class="top-nav-theme icon-btn" type="button" :title="`${themeToggleLabel} mode`" @click="toggleTheme">
          <span class="material-symbols-rounded" aria-hidden="true">{{ themeIcon }}</span>
          <span class="sr-only">{{ themeToggleLabel }} mode</span>
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
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--card) 82%, var(--nav-mix) 18%), color-mix(in srgb, var(--card) 90%, var(--nav-mix) 10%));
  backdrop-filter: blur(10px);
  box-shadow: 0 8px 22px rgba(14, 32, 72, 0.15);
}

.top-nav-inner {
  max-width: 1020px;
  margin: 0 auto;
  padding: 0.8rem 1rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.brand-link {
  text-decoration: none;
  color: var(--ink-1);
  font-size: 1.1rem;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  text-shadow: 2px 2px 0 rgba(15, 47, 140, 0.08);
}

.top-nav-links {
  display: flex;
  align-items: center;
  gap: 0.42rem;
}

.top-nav-link {
  text-decoration: none;
  padding: 0.38rem 0.72rem;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--brand-2) 28%, var(--line) 72%);
  background: color-mix(in srgb, var(--card) 90%, #edf5ff 10%);
  color: var(--brand-1);
  font-weight: 760;
  letter-spacing: 0.01em;
  transition: box-shadow 0.16s ease, background 0.16s ease, border-color 0.16s ease, transform 0.12s ease;
}

.top-nav-link:hover {
  border-color: color-mix(in srgb, var(--brand-2) 48%, var(--line) 52%);
  background: color-mix(in srgb, var(--brand-2) 10%, var(--card) 90%);
  transform: translateY(-1px);
}

.top-nav-link:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--brand-2) 50%, white 50%);
  outline-offset: 1px;
}

.top-nav-link.router-link-active {
  color: #fff;
  border-color: color-mix(in srgb, #0f4f99 75%, var(--brand-1) 25%);
  background: linear-gradient(130deg, #0f4f99, var(--brand-1));
  box-shadow: 0 8px 18px rgba(30, 136, 229, 0.3);
}

.top-nav-user-menu {
  position: relative;
  display: inline-flex;
  align-items: center;
}

.top-nav-user-trigger {
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--brand-2) 30%, var(--line) 70%);
  background: color-mix(in srgb, var(--card) 88%, #eef5ff 12%);
  color: var(--ink-1);
  padding: 0.34rem 0.58rem;
  display: inline-flex;
  align-items: center;
  gap: 0.18rem;
  font-size: 0.84rem;
  font-weight: 760;
  font-family: "Space Mono", ui-monospace, monospace;
}

.top-nav-user-avatar {
  width: 1.35rem;
  height: 1.35rem;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0.72rem;
  font-weight: 800;
  color: #fff;
  background: linear-gradient(130deg, #0f4f99, var(--brand-1));
  border: 1px solid color-mix(in srgb, var(--brand-2) 44%, #0f4f99 56%);
  box-shadow: 0 4px 10px rgba(30, 136, 229, 0.26);
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
  border: 1px solid color-mix(in srgb, var(--brand-2) 26%, var(--line) 74%);
  background: color-mix(in srgb, var(--card) 94%, #f0f6ff 6%);
  box-shadow: 0 12px 26px rgba(12, 28, 63, 0.24);
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
  display: block;
  width: 100%;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--line) 88%, var(--brand-1) 12%);
  background: color-mix(in srgb, var(--card) 90%, #edf4ff 10%);
  color: var(--ink-1);
  text-align: left;
  text-decoration: none;
  padding: 0.42rem 0.56rem;
  font-weight: 700;
  cursor: pointer;
}

.top-nav-user-action:hover {
  border-color: color-mix(in srgb, var(--brand-2) 46%, var(--line) 54%);
  background: color-mix(in srgb, var(--brand-2) 14%, var(--card) 86%);
}

.top-nav-current {
  font-size: 0.83rem;
  color: var(--ink-2);
  font-family: "Space Mono", ui-monospace, monospace;
}

.top-nav-theme {
  border: 1px solid color-mix(in srgb, var(--brand-2) 32%, var(--line) 68%);
  background: color-mix(in srgb, var(--brand-2) 14%, var(--card) 86%);
  color: var(--ink-1);
  border-radius: 999px;
  padding: 0.34rem 0.62rem;
  font-size: 0.82rem;
  font-weight: 700;
}

@media (max-width: 900px) {
  .top-nav-current {
    display: none;
  }
}
</style>
