import type { ConfirmOptions } from '../stores/confirms'
import { useConfirmsStore } from '../stores/confirms'

export function useConfirm() {
  const store = useConfirmsStore()

  return {
    ask(options: ConfirmOptions = {}): Promise<boolean> {
      return store.ask(options)
    },
    cancel() {
      store.cancel()
    },
    confirm() {
      store.confirm()
    },
  }
}
