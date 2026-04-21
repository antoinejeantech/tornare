import './lib/pwa'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from './stores/auth'
import App from './App.vue'
import { i18n } from './i18n'
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
import OnboardingPage from './pages/OnboardingPage.vue'
import DiscordGuildPage from './pages/DiscordGuildPage.vue'
import VerifyEmailPage from './pages/VerifyEmailPage.vue'
import VerifyEmailPendingPage from './pages/VerifyEmailPendingPage.vue'
import ForgotPasswordPage from './pages/ForgotPasswordPage.vue'
import ResetPasswordPage from './pages/ResetPasswordPage.vue'

const pinia = createPinia()

const router = createRouter({
	history: createWebHistory(),
	scrollBehavior: () => ({ top: 0 }),
	routes: [
		{ path: '/', name: 'home', component: HomePage, meta: { title: 'Tornare' } },
		{ path: '/events', name: 'events', component: EventsPage, meta: { title: 'Events | Tornare' } },
		{ path: '/about', name: 'about', component: AboutPage, meta: { title: 'About | Tornare' } },
		{ path: '/news', name: 'news', component: NewsPage, meta: { title: 'News | Tornare' } },
		{ path: '/privacy', name: 'privacy', component: PrivacyPage, meta: { title: 'Privacy Policy | Tornare' } },
		{ path: '/terms', name: 'terms', component: TermsPage, meta: { title: 'Terms Of Service | Tornare' } },
		{ path: '/faq', name: 'faq', component: FaqPage, meta: { title: 'FAQ | Tornare' } },
		{ path: '/support', name: 'support', component: SupportPage, meta: { title: 'Support | Tornare' } },
		{ path: '/auth', redirect: '/login' },
		{ path: '/login', name: 'login', component: AuthPage, meta: { title: 'Sign In | Tornare' } },
		{ path: '/register', name: 'register', component: AuthPage, meta: { title: 'Create Account | Tornare' } },
		{ path: '/onboarding', name: 'onboarding', component: OnboardingPage, meta: { title: 'Set up your account | Tornare' } },
		{ path: '/verify-email', name: 'verify-email', component: VerifyEmailPage, meta: { title: 'Verify Email | Tornare' } },
		{ path: '/verify-email/pending', name: 'verify-email-pending', component: VerifyEmailPendingPage, meta: { title: 'Check Your Email | Tornare' } },
		{ path: '/forgot-password', name: 'forgot-password', component: ForgotPasswordPage, meta: { title: 'Forgot Password | Tornare' } },
		{ path: '/reset-password', name: 'reset-password', component: ResetPasswordPage, meta: { title: 'Reset Password | Tornare' } },
		{ path: '/discord', name: 'discord-guild', component: DiscordGuildPage, meta: { title: 'Discord Bot | Tornare', requiresAuth: true } },
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

  if ((to.name === 'login' || to.name === 'register' || to.name === 'verify-email-pending') && authStore.isAuthenticated) {
    return { name: 'events' }
  }

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    return { name: 'login', query: { redirect: to.fullPath } }
  }

  return true
})

router.afterEach((to) => {
	document.title = typeof to.meta.title === 'string' ? to.meta.title : 'Tornare'
})

createApp(App).use(pinia).use(router).use(i18n).mount('#app')
