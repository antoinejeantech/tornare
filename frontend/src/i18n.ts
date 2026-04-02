import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import fr from './locales/fr.json'

const LOCALE_STORAGE_KEY = 'tornare_locale'

function detectLocale(): string {
  const stored = typeof window !== 'undefined'
    ? window.localStorage.getItem(LOCALE_STORAGE_KEY)
    : null
  if (stored === 'fr' || stored === 'en') {
    return stored
  }
  const browser = typeof navigator !== 'undefined'
    ? navigator.language.slice(0, 2).toLowerCase()
    : 'en'
  return browser === 'fr' ? 'fr' : 'en'
}

export const i18n = createI18n({
  legacy: false,
  locale: detectLocale(),
  fallbackLocale: 'en',
  messages: { en, fr },
})

export function setLocale(locale: 'en' | 'fr') {
  (i18n.global.locale as unknown as { value: string }).value = locale
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(LOCALE_STORAGE_KEY, locale)
    document.documentElement.setAttribute('lang', locale)
  }
}

export function getLocale(): string {
  return (i18n.global.locale as unknown as { value: string }).value
}
