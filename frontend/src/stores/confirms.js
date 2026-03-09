import { defineStore } from 'pinia'

let nextId = 1

function normalizeTone(value) {
  const tone = String(value || '').toLowerCase()
  if (tone === 'danger' || tone === 'warning') {
    return tone
  }
  return 'default'
}

export const useConfirmsStore = defineStore('confirms', {
  state: () => ({
    current: null,
    queue: [],
  }),
  actions: {
    ask(payload = {}) {
      return new Promise((resolve) => {
        const request = {
          id: nextId,
          title: payload.title || 'Please confirm',
          message: payload.message || '',
          confirmText: payload.confirmText || 'Confirm',
          cancelText: payload.cancelText || 'Cancel',
          tone: normalizeTone(payload.tone),
          resolve,
        }

        nextId += 1

        if (!this.current) {
          this.current = request
          return
        }

        this.queue.push(request)
      })
    },
    respond(confirmed) {
      if (!this.current) {
        return
      }

      const active = this.current
      this.current = null
      active.resolve(Boolean(confirmed))

      if (this.queue.length > 0) {
        const [next, ...rest] = this.queue
        this.current = next
        this.queue = rest
      }
    },
    cancel() {
      this.respond(false)
    },
    confirm() {
      this.respond(true)
    },
    clear() {
      if (this.current) {
        this.current.resolve(false)
      }

      for (const queued of this.queue) {
        queued.resolve(false)
      }

      this.current = null
      this.queue = []
    },
  },
})
