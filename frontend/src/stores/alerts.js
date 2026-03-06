import { defineStore } from 'pinia'

let nextId = 1

export const useAlertsStore = defineStore('alerts', {
  state: () => ({
    items: [],
  }),
  actions: {
    push(payload) {
      const id = nextId
      nextId += 1

      const item = {
        id,
        type: payload.type || 'info',
        title: payload.title || '',
        message: payload.message || '',
      }

      this.items.push(item)

      const duration = Number(payload.duration)
      const ttl = Number.isFinite(duration) ? duration : 2400
      if (ttl > 0) {
        window.setTimeout(() => {
          this.remove(id)
        }, ttl)
      }

      return id
    },
    remove(id) {
      this.items = this.items.filter((item) => item.id !== id)
    },
    clear() {
      this.items = []
    },
  },
})
