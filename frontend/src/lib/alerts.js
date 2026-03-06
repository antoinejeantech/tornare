import { useAlertsStore } from '../stores/alerts'

export function useAlert() {
  const store = useAlertsStore()

  return {
    success(message, options = {}) {
      return store.push({ type: 'success', message, ...options })
    },
    error(message, options = {}) {
      return store.push({ type: 'error', message, duration: 4200, ...options })
    },
    info(message, options = {}) {
      return store.push({ type: 'info', message, ...options })
    },
    warning(message, options = {}) {
      return store.push({ type: 'warning', message, duration: 3200, ...options })
    },
    dismiss(id) {
      store.remove(id)
    },
    clearAll() {
      store.clear()
    },
  }
}
