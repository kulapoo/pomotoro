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
 * no IPC round-trip. `fetchTimer` is still called on the four task events
 * because the orchestrators (`switch_active_task`, `reset_task`, etc.) do
 * not emit `timer:*` events after `load_state` (documented gap,
 * docs/superpowers/specs/2026-06-27-task-switch-resets-timer-design.md).
 * Non-calling windows therefore need an explicit timer re-read to stay
 * in sync with the new task's bound timer.
 *
 * Scope rules:
 *  - Timer events are global: the timer must keep reconciling even while
 *    the Timer page is unmounted.
 *  - `task:task_completed` and `task:task_reset` may target a non-active
 *    task; the conditional setter (`applyTaskIfActiveForId`) leaves
 *    `activeTask` untouched in that case.
 *  - `timer:status_changed` is covered by the start/pause/resume/reset
 *    family below (no separate handler needed here).
 */
export function useEventBus(): void {
  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const applyTick = useTimerStore((s) => s.applyTick)
  const applyTimerState = useTimerStore((s) => s.applyTimerState)
  const applyActiveTask = useTaskStore((s) => s.applyActiveTask)
  const applyTaskIfActiveForId = useTaskStore((s) => s.applyTaskIfActiveForId)

  useEffect(() => {
    fetchTimer()

    const unlisteners: Array<Promise<UnlistenFn>> = [
      // Real-time countdown; pure local state update.
      onEvent(events.timerTick, applyTick),

      // Timer lifecycle: payload is `TimerStateData`; preserve task_id.
      onEvent(events.timerPhaseCompleted, applyTimerState),
      onEvent(events.timerReset, applyTimerState),
      onEvent(events.timerPaused, applyTimerState),
      onEvent(events.timerStarted, applyTimerState),
      onEvent(events.timerResumed, applyTimerState),

      // Task events: direct-map the embedded Task; re-fetch the timer
      // because the timer state is bound to the active task and the
      // orchestrator does not emit a timer:* event after load_state.
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
      onEvent(events.taskAutoAdvanced, (payload) => {
        applyActiveTask(payload.to_task)
        fetchTimer()
        toast.success('Switched to next task')
      }),

      // Screen blocker: show the focus-enforcement overlay.
      onEvent(events.screenBlockerActivate, (payload) => {
        useScreenBlockerStore.getState().activate(payload.message)
      }),
    ]

    return () => {
      for (const p of unlisteners) void p.then((fn) => fn())
    }
  }, [fetchTimer, applyTick, applyTimerState, applyActiveTask, applyTaskIfActiveForId])
}
