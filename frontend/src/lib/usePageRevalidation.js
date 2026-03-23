import { onBeforeUnmount, onMounted } from 'vue'

/**
 * Calls `callback` when the page becomes visible after having been hidden for
 * at least `thresholdMs` milliseconds (default 30 s). Registers/unregisters
 * the listener automatically via onMounted / onBeforeUnmount, so it must be
 * called inside a component's <script setup> or setup().
 *
 * Usage:
 *   import { usePageRevalidation } from '../lib/usePageRevalidation'
 *   usePageRevalidation(() => loadMyData())
 */
export function usePageRevalidation(callback, thresholdMs = 30_000) {
  let hiddenAt = 0

  function onVisibilityChange() {
    if (document.visibilityState === 'hidden') {
      hiddenAt = Date.now()
    } else if (document.visibilityState === 'visible' && hiddenAt > 0) {
      const hiddenMs = Date.now() - hiddenAt
      hiddenAt = 0
      if (hiddenMs >= thresholdMs) {
        callback()
      }
    }
  }

  onMounted(() => {
    document.addEventListener('visibilitychange', onVisibilityChange)
  })

  onBeforeUnmount(() => {
    document.removeEventListener('visibilitychange', onVisibilityChange)
  })
}
