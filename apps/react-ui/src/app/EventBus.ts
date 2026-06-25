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
 *  - `task:list_updated` and `task:task_completed` reload the *active task*
 *    (not the full list) so actions triggered outside the UI — the tray menu,
 *    auto-cycling, background use cases — keep the Timer page in sync. The full
 *    task-list reload is still scoped to the Tasks page via
 *    `useTasksEventBus`.
 *  - `timer:status_changed` covers start/resume/reset/expiry transitions
 *    regardless of trigger source (it is what the tray-emitted transitions
 *    surface as; `timer:timer_started` / `timer:timer_resumed` are dead).
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

    const reload = () => {
      window.setTimeout(() => {
        reloadTimer()
        reloadActiveTask()
      }, 1000)
    }

    const unlisteners: Array<Promise<UnlistenFn>> = [
      // Real-time countdown; pure local state update, no network.
      onEvent(events.timerTick, applyTick),
      onEvent(events.timerPhaseCompleted, reload),
      onEvent(events.timerReset, reloadTimer),
      onEvent(events.timerPaused, reloadTimer),
      onEvent(events.timerStarted, reloadTimer),
      onEvent(events.timerResumed, reloadTimer),
      onEvent(events.taskActiveChanged, reload),

      onEvent(events.taskAutoAdvanced, () => {
        reload()
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
