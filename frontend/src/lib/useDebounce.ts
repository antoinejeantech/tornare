import { onBeforeUnmount } from 'vue'

/**
 * Returns a `debounced` scheduler and a `cancel` function for a given delay.
 * The pending timer is automatically cancelled on component unmount.
 *
 * Usage:
 *   const { debounced, cancel } = useDebounce(300)
 *   debounced(() => doSomething())  // resets the timer on every call
 *   cancel()                        // clears without running
 */
export function useDebounce(delay: number) {
  let timer: ReturnType<typeof setTimeout> | null = null

  function debounced(fn: () => void): void {
    if (timer) clearTimeout(timer)
    timer = setTimeout(fn, delay)
  }

  function cancel(): void {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
  }

  onBeforeUnmount(cancel)

  return { debounced, cancel }
}
