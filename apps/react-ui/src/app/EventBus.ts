import { useEffect } from 'react'
import { toast } from 'sonner'
import { onEvent, events } from '@/lib/tauri'
import { createBatchedLoader } from '@/lib/async'
import { useTimerStore } from '@/pages/timer/useTimer'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { useTaskStore } from '@/pages/tasks/useTasks'
import { useScreenBlockerStore } from '@/app/useScreenBlocker'

/**
 * Global, always-on backend event subscriptions.
 *
 * Scope rules:
 *  - Timer events are global: the timer must keep reconciling even while the
 *    Timer page is unmounted, so navigating back always shows fresh state.
 *  - Task events that flip the active task also refresh the timer here,
 *    because the timer is bound to the active task.
 *  - Task-list events (`task:completed`, `task:progress_updated`,
 *    `task:list_updated`) are intentionally NOT global — they only matter on
 *    the Tasks page and are wired there via `useTasksEventBus`, avoiding
 *    unnecessary `loadTasks` round-trips.
 *
 * All re-fetches are coalesced through {@link createBatchedLoader} so a burst
 * of related events collapses into one fetch and overlapping fetches cannot
 * leave the store in a stale state.
 */
export function useEventBus(): void {
  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const applyTick = useTimerStore((s) => s.applyTick)
  const loadActiveTask = useTaskStore((s) => s.loadActiveTask)

  useEffect(() => {
    const reloadTimer = createBatchedLoader(() => fetchTimer())
    const reloadActiveTask = createBatchedLoader(() => loadActiveTask())

    const unlisteners: Array<Promise<UnlistenFn>> = [
      // Real-time countdown; pure local state update, no network.
      onEvent(events.timerTick, applyTick),
      // Authoritative re-fetch after any timer transition.
      onEvent(events.timerPhaseCompleted, () => {
        window.setTimeout(() => {
          reloadTimer()
          reloadActiveTask()
        }, 200)
      }),
      onEvent(events.timerReset, reloadTimer),
      onEvent(events.timerPaused, reloadTimer),
      onEvent(events.timerResumed, reloadTimer),
      // Active task changed — timer context follows it.
      onEvent(events.taskActiveChanged, () => {
        window.setTimeout(() => {
          reloadTimer()
          reloadActiveTask()
        }, 500)
      }),
      onEvent(events.taskAutoAdvanced, () => {
        window.setTimeout(() => {
          reloadTimer()
          reloadActiveTask()
        }, 500)
        toast.success('Switched to next incomplete task')
      }),
      // Screen blocker: show the focus-enforcement overlay when a work/break
      // phase expires and blocking is enabled for that phase.
      onEvent(events.screenBlockerActivate, (payload) => {
        useScreenBlockerStore.getState().activate(payload.message)
      }),
    ]

    return () => {
      for (const p of unlisteners) void p.then((fn) => fn())
    }
  }, [fetchTimer, applyTick, loadActiveTask])
}
