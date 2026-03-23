import { defineStore } from 'pinia'

export type ConfirmTone = 'default' | 'danger' | 'warning'

export interface ConfirmOptions {
  title?: string
  message?: string
  confirmText?: string
  cancelText?: string
  tone?: string
}

interface ConfirmRequest {
  id: number
  title: string
  message: string
  confirmText: string
  cancelText: string
  tone: ConfirmTone
  resolve: (value: boolean) => void
}

let nextId = 1

function normalizeTone(value: string | undefined | null): ConfirmTone {
  const tone = String(value || '').toLowerCase()
  if (tone === 'danger' || tone === 'warning') {
    return tone
  }
  return 'default'
}

export const useConfirmsStore = defineStore('confirms', {
  state: () => ({
    current: null as ConfirmRequest | null,
    queue: [] as ConfirmRequest[],
  }),
  actions: {
    ask(payload: ConfirmOptions = {}): Promise<boolean> {
      return new Promise((resolve) => {
        const request: ConfirmRequest = {
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
    respond(confirmed: boolean): void {
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
    cancel(): void {
      this.respond(false)
    },
    confirm(): void {
      this.respond(true)
    },
    clear(): void {
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
