/**
 * Returns helpers for guarding stale async requests.
 * Call `next()` before each request to get a unique ID, then check
 * `isCurrent(id)` after the await to discard out-of-order responses.
 * Call `invalidate()` to make all in-flight IDs stale without issuing a new one.
 *
 * Usage:
 *   const { next, isCurrent, invalidate } = useRequestSequence()
 *   const id = next()
 *   const data = await fetch(...)
 *   if (!isCurrent(id)) return
 */
export function useRequestSequence() {
  let current = 0

  function next(): number {
    return ++current
  }

  function isCurrent(id: number): boolean {
    return id === current
  }

  function invalidate(): void {
    current++
  }

  return { next, isCurrent, invalidate }
}
