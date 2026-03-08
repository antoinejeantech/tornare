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
import JoinEventPage from './pages/JoinEventPage.vue'
import ProfilePage from './pages/ProfilePage.vue'

const pinia = createPinia()

const router = createRouter({
	history: createWebHistory(),
	routes: [
		{ path: '/', name: 'home', component: HomePage, meta: { title: 'Tornare' } },
		{ path: '/events', name: 'events', component: EventsPage, meta: { title: 'Events | Tornare' } },
		{ path: '/about', name: 'about', component: AboutPage, meta: { title: 'About | Tornare' } },
		{ path: '/news', name: 'news', component: NewsPage, meta: { title: 'News | Tornare' } },
		{ path: '/auth', name: 'auth', component: AuthPage, meta: { title: 'Sign In | Tornare' } },
		{ path: '/events/:id', name: 'event', component: EventPage, meta: { title: 'Event Setup | Tornare' } },
		{ path: '/join/:token', name: 'join-event', component: JoinEventPage, meta: { title: 'Join Event | Tornare' } },
		{ path: '/profiles/:id', name: 'profile', component: ProfilePage, meta: { title: 'Profile | Tornare' } },
		{ path: '/matches/:id', name: 'match', component: MatchPage, meta: { requiresAuth: true, title: 'Match | Tornare' } }
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

router.afterEach((to) => {
	document.title = typeof to.meta.title === 'string' ? to.meta.title : 'Tornare'
})

createApp(App).use(pinia).use(router).mount('#app')
