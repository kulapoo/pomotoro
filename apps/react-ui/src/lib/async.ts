/**
 * Returns a scheduler that coalesces rapid successive invocations of `fn`
 * into a single call and never races overlapping calls.
 *
 * Two correctness guarantees:
 *  1. Burst coalescing — multiple calls within the same microtask collapse
 *     into one execution (Tauri often emits several related events at once,
 *     e.g. auto-advance fires `task:completed` + `task:auto_advanced`).
 *  2. Trailing re-run — if a call arrives while `fn` is still in flight, the
 *     scheduler re-runs `fn` once after it settles, so the last-writer
 *     always wins and overlapping fetches can never leave the store stale.
 */
export function createBatchedLoader(fn: () => Promise<unknown>): () => void {
  let scheduled = false
  let inFlight: Promise<unknown> | null = null
  let dirty = false

  const run = async (): Promise<void> => {
    scheduled = false
    inFlight = fn()
    try {
      await inFlight
    } finally {
      inFlight = null
    }
    if (dirty) {
      dirty = false
      void run()
    }
  }

  const scheduleRun = (): void => {
    if (scheduled) return
    scheduled = true
    queueMicrotask(() => void run())
  }

  return (): void => {
    if (inFlight) {
      dirty = true
      return
    }
    scheduleRun()
  }
}
