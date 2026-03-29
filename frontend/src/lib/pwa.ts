import { ref } from 'vue'
import { registerSW } from 'virtual:pwa-register'

/**
 * Reactive flag set to true when a new service worker is waiting to activate.
 * Consumed by PwaUpdatePrompt to show an update toast.
 */
export const needRefresh = ref(false)

const updateSW = registerSW({
  onNeedRefresh() {
    needRefresh.value = true
  },
})

/**
 * Tells the waiting service worker to skip waiting, then reloads the page.
 * Called by PwaUpdatePrompt when the user confirms the update.
 */
export async function applyUpdate(): Promise<void> {
  await updateSW(true)
}
