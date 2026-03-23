import type { AlertPushOptions } from '../stores/alerts'
import { useAlertsStore } from '../stores/alerts'

export function useAlert() {
  const store = useAlertsStore()

  return {
    success(message: string, options: AlertPushOptions = {}) {
      return store.push({ type: 'success', message, ...options })
    },
    error(message: string, options: AlertPushOptions = {}) {
      return store.push({ type: 'error', message, duration: 4200, ...options })
    },
    info(message: string, options: AlertPushOptions = {}) {
      return store.push({ type: 'info', message, ...options })
    },
    warning(message: string, options: AlertPushOptions = {}) {
      return store.push({ type: 'warning', message, duration: 3200, ...options })
    },
    dismiss(id: number) {
      store.remove(id)
    },
    clearAll() {
      store.clear()
    },
  }
}
