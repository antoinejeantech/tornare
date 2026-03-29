import './lib/pwa'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from './stores/auth'
import App from './App.vue'
import './styles.css'
import HomePage from './pages/HomePage.vue'
import EventsPage from './pages/EventsPage.vue'
import EventPage from './pages/EventPage.vue'
import AboutPage from './pages/AboutPage.vue'
import NewsPage from './pages/NewsPage.vue'
import AuthPage from './pages/AuthPage.vue'
import AuthCallbackPage from './pages/AuthCallbackPage.vue'
import JoinEventPage from './pages/JoinEventPage.vue'
import ProfilePage from './pages/ProfilePage.vue'
import PrivacyPage from './pages/PrivacyPage.vue'
import TermsPage from './pages/TermsPage.vue'
import FaqPage from './pages/FaqPage.vue'
import SupportPage from './pages/SupportPage.vue'
import NotFoundPage from './pages/NotFoundPage.vue'

const pinia = createPinia()

const router = createRouter({
	history: createWebHistory(),
	routes: [
		{ path: '/', name: 'home', component: HomePage, meta: { title: 'Tornare' } },
		{ path: '/events', name: 'events', component: EventsPage, meta: { title: 'Events | Tornare' } },
		{ path: '/about', name: 'about', component: AboutPage, meta: { title: 'About | Tornare' } },
		{ path: '/news', name: 'news', component: NewsPage, meta: { title: 'News | Tornare' } },
		{ path: '/privacy', name: 'privacy', component: PrivacyPage, meta: { title: 'Privacy Policy | Tornare' } },
		{ path: '/terms', name: 'terms', component: TermsPage, meta: { title: 'Terms Of Service | Tornare' } },
		{ path: '/faq', name: 'faq', component: FaqPage, meta: { title: 'FAQ | Tornare' } },
		{ path: '/support', name: 'support', component: SupportPage, meta: { title: 'Support | Tornare' } },
		{ path: '/auth', name: 'auth', component: AuthPage, meta: { title: 'Sign In | Tornare' } },
		{ path: '/auth/callback', name: 'auth-callback', component: AuthCallbackPage, meta: { title: 'Signing in… | Tornare' } },
		{ path: '/events/:id', name: 'event', component: EventPage, meta: { title: 'Event Setup | Tornare' } },
		{ path: '/join/:token', name: 'join-event', component: JoinEventPage, meta: { title: 'Join Event | Tornare' } },
		{ path: '/profiles/:id', name: 'profile', component: ProfilePage, meta: { title: 'Profile | Tornare' } },
		{ path: '/:pathMatch(.*)*', name: 'not-found', component: NotFoundPage, meta: { title: '404 | Tornare' } }
	]
})

router.beforeEach(async (to) => {
  const authStore = useAuthStore(pinia)
  await authStore.initialize()
  // Re-sync store state with localStorage on every navigation. This catches cases where
  // tryRefreshSession() updated the tokens in storage without going through the auth store
  // actions, keeping authStore.isAuthenticated and the module-level access token consistent.
  authStore.syncTokensFromStorage()

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
