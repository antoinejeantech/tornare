import { useConfirmsStore } from '../stores/confirms'

export function useConfirm() {
  const store = useConfirmsStore()

  return {
    ask(options = {}) {
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
