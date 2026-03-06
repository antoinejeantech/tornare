<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { RouterLink } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
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
        <RouterLink v-if="!authStore.isAuthenticated" class="top-nav-link" to="/auth">Login</RouterLink>
        <div v-else class="top-nav-user-menu" tabindex="0">
          <button class="top-nav-user-trigger" type="button">
            <span class="top-nav-user-avatar" aria-hidden="true">{{ authInitial }}</span>
            <span>{{ authLabel }}</span>
            <span class="material-symbols-rounded" aria-hidden="true">expand_more</span>
          </button>
          <div class="top-nav-user-dropdown" role="menu" aria-label="User menu">
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
