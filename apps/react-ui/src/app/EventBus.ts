import { useEffect } from 'react'
import { toast } from 'sonner'
import { onEvent, events } from '@/lib/tauri'
import { shortId } from '@/lib/id'
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
 *  - `applyTimer` (replaces the whole Timer, including task_id) is used
 *   for the four timer UI events (`timer:timer_started/paused/reset/
 *   resumed`, whose payloads now carry task_id) and for
 *   `task:auto_advanced` where the cycle swaps the bound task.
 *  - `applyTimerState` (preserves task_id) is used only by
 *   `timer:phase_completed`, whose payload carries a separate `timer`
 *   field and whose bound task is unchanged.
 */
export function useEventBus(): void {
  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const applyTick = useTimerStore((s) => s.applyTick)
  const applyTimerState = useTimerStore((s) => s.applyTimerState)
  const applyTimer = useTimerStore((s) => s.applyTimer)
  const applyActiveTask = useTaskStore((s) => s.applyActiveTask)
  const applyTaskIfActiveForId = useTaskStore((s) => s.applyTaskIfActiveForId)
  const loadActiveTask = useTaskStore((s) => s.loadActiveTask)

  useEffect(() => {
    fetchTimer()
    loadActiveTask()

    const unlisteners: Array<Promise<UnlistenFn>> = [
      onEvent(events.timerTick, applyTick),

      onEvent(events.timerPhaseCompleted, (payload) => {
        applyTimerState(payload.timer)
        if (payload.task) {
          applyTaskIfActiveForId(payload.task.id, payload.task)
        }
      }),

      onEvent(events.timerReset, applyTimer),
      onEvent(events.timerPaused, applyTimer),
      onEvent(events.timerStarted, applyTimer),
      onEvent(events.timerResumed, applyTimer),

      onEvent(events.taskActiveChanged, (payload) => {
        if (payload) {
          applyActiveTask(payload.task)
        }
      }),
      onEvent(events.taskReset, (payload) => {
        applyTaskIfActiveForId(payload.task_id, payload.task)
        fetchTimer()
      }),

      onEvent(events.taskAutoAdvanced, (payload) => {
        applyActiveTask(payload.to_task)
        applyTimer(payload.timer)
        toast.success(
          `Switched to "${payload.to_task.name}" (${shortId(payload.to_task.id)})`,
        )
      }),

      onEvent(events.taskCompleted, (payload) => {
        window.setTimeout(() => {
          applyTaskIfActiveForId(payload.task_id, payload.task, {
            completed_at: payload.completed_at,
          })
        }, 300)
      }),

      onEvent(events.screenBlockerActivate, (payload) => {
        useScreenBlockerStore.getState().activate(payload.message)
      }),
    ]

    return () => {
      for (const p of unlisteners) void p.then((fn) => fn())
    }
  }, [
    fetchTimer,
    loadActiveTask,
    applyTick,
    applyTimerState,
    applyTimer,
    applyActiveTask,
    applyTaskIfActiveForId,
  ])
}
