import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from './stores/auth'
import App from './App.vue'
import './styles.css'
import HomePage from './pages/HomePage.vue'
import EventsPage from './pages/EventsPage.vue'
import MatchPage from './pages/MatchPage.vue'
import EventPage from './pages/EventPage.vue'
import AboutPage from './pages/AboutPage.vue'
import NewsPage from './pages/NewsPage.vue'
import AuthPage from './pages/AuthPage.vue'

const pinia = createPinia()

const router = createRouter({
	history: createWebHistory(),
	routes: [
		{ path: '/', name: 'home', component: HomePage },
		{ path: '/events', name: 'events', component: EventsPage, meta: { requiresAuth: true } },
		{ path: '/about', name: 'about', component: AboutPage },
		{ path: '/news', name: 'news', component: NewsPage },
		{ path: '/auth', name: 'auth', component: AuthPage },
		{ path: '/events/:id', name: 'event', component: EventPage, meta: { requiresAuth: true } },
		{ path: '/matches/:id', name: 'match', component: MatchPage, meta: { requiresAuth: true } }
	]
})

router.beforeEach(async (to) => {
  const authStore = useAuthStore(pinia)
  await authStore.initialize()

  if (to.name === 'auth' && authStore.isAuthenticated) {
    return { name: 'events' }
  }

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    return { name: 'auth', query: { redirect: to.fullPath } }
  }

  return true
})

createApp(App).use(pinia).use(router).mount('#app')
