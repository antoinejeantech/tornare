import { defineStore } from 'pinia'

export type AlertType = 'success' | 'error' | 'info' | 'warning'

export interface AlertItem {
  id: number
  type: AlertType
  title: string
  message: string
}

export interface AlertPushOptions {
  type?: AlertType
  title?: string
  duration?: number
}

interface AlertPushPayload extends AlertPushOptions {
  message: string
}

let nextId = 1

export const useAlertsStore = defineStore('alerts', {
  state: () => ({
    items: [] as AlertItem[],
  }),
  actions: {
    push(payload: AlertPushPayload): number {
      const id = nextId
      nextId += 1

      const item: AlertItem = {
        id,
        type: payload.type || 'info',
        title: payload.title || '',
        message: payload.message || '',
      }

      this.items.push(item)

      const duration = Number(payload.duration)
      const ttl = Number.isFinite(duration) ? duration : 5000
      if (ttl > 0) {
        window.setTimeout(() => {
          this.remove(id)
        }, ttl)
      }

      return id
    },
    remove(id: number): void {
      this.items = this.items.filter((item) => item.id !== id)
    },
    clear(): void {
      this.items = []
    },
  },
})
