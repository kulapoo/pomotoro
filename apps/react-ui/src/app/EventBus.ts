import { useEffect } from 'react'
import { toast } from 'sonner'
import { onEvent, events } from '@/lib/tauri'
import { useTimerStore } from '@/pages/timer/useTimer'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { useTaskStore } from '@/pages/tasks/useTasks'
import { useScreenBlockerStore } from '@/app/useScreenBlocker'

/**
 * Global, always-on backend event subscriptions.
 *
 * Each handler maps its payload directly into the relevant store slice —
 * no IPC round-trip except `task:list_updated` (whose backend payload
 * shape is heterogeneous across 7 emitters and cannot be direct-mapped).
 *
 * `fetchTimer` is still called on three task events (`active_changed`,
 * `task_completed`, `task_reset`) because their payloads do not carry
 * timer state and the orchestrators do not emit `timer:*` after
 * `load_state` (documented gap,
 * docs/superpowers/specs/2026-06-27-task-switch-resets-timer-design.md).
 *
 * Scope rules:
 *  - Timer events are global: the timer must keep reconciling even while
 *   the Timer page is unmounted.
 *  - `task:task_completed` and `task:task_reset` may target a non-active
 *   task; the conditional setter (`applyTaskIfActiveForId`) leaves
 *   `activeTask` untouched in that case.
 *  - `applyTimerState` (preserves task_id) is used for events where the
 *   bound task is unchanged; `applyTimer` (replaces whole timer) is used
 *   for `task:auto_advanced` where the cycle swaps the bound task.
 */
export function useEventBus(): void {
  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const applyTick = useTimerStore((s) => s.applyTick)
  const applyTimerState = useTimerStore((s) => s.applyTimerState)
  const applyTimer = useTimerStore((s) => s.applyTimer)
  const applyActiveTask = useTaskStore((s) => s.applyActiveTask)
  const applyTaskIfActiveForId = useTaskStore((s) => s.applyTaskIfActiveForId)

  useEffect(() => {
    fetchTimer()

    const unlisteners: Array<Promise<UnlistenFn>> = [
      // Real-time countdown; pure local state update.
      onEvent(events.timerTick, applyTick),

      // PHASE_COMPLETED now carries { timer, task } (envelope). The inner
      // `timer` is a bare TimerStateData — task is unchanged, so we
      // preserve the existing task_id via applyTimerState.
      onEvent(events.timerPhaseCompleted, (payload) => {
        applyTimerState(payload.timer)
        applyTaskIfActiveForId(payload.task.id, payload.task)
      }),

      // Timer lifecycle: payload is bare TimerStateData; preserve task_id.
      onEvent(events.timerReset, applyTimerState),
      onEvent(events.timerPaused, applyTimerState),
      onEvent(events.timerStarted, applyTimerState),
      onEvent(events.timerResumed, applyTimerState),

      // Task events: direct-map the embedded Task; re-fetch the timer
      // for the three events whose payload does not carry timer state.
      onEvent(events.taskActiveChanged, (payload) => {
        if (payload) {
          applyActiveTask(payload.task)
          fetchTimer()
        }
      }),
      onEvent(events.taskCompleted, (payload) => {
        applyTaskIfActiveForId(payload.task_id, payload.task)
        fetchTimer()
        toast.success('Task completed!')
      }),
      onEvent(events.taskReset, (payload) => {
        applyTaskIfActiveForId(payload.task_id, payload.task)
        fetchTimer()
        toast.info('Task progress reset')
      }),

      // AUTO_ADVANCED carries the new task AND the full new Timer (with
      // new task_id) — applyTimer replaces both fields. No IPC.
      onEvent(events.taskAutoAdvanced, (payload) => {
        applyActiveTask(payload.to_task)
        applyTimer(payload.timer)
        toast.success('Switched to next task')
      }),

      // PROGRESS_UPDATED carries a bare Task — the only signal for
      // current_sessions increments when no cycle occurs.
      onEvent(events.taskProgressUpdated, (task) => {
        applyTaskIfActiveForId(task.id, task)
      }),

      // LIST_UPDATED payload is heterogeneous across 7 emitters — treat
      // as opaque dirty signal and reload both slices. `loadActiveTask`
      // is needed because some mutations (e.g. edit active task's name)
      // emit only this event.
      onEvent(events.taskListUpdated, () => {
        const s = useTaskStore.getState()
        void s.loadTasks()
        void s.loadActiveTask()
      }),

      // Screen blocker: show the focus-enforcement overlay.
      onEvent(events.screenBlockerActivate, (payload) => {
        useScreenBlockerStore.getState().activate(payload.message)
      }),
    ]

    return () => {
      for (const p of unlisteners) void p.then((fn) => fn())
    }
  }, [fetchTimer, applyTick, applyTimerState, applyTimer, applyActiveTask, applyTaskIfActiveForId])
}
