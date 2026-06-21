import { useEffect } from 'react'
import { toast } from 'sonner'
import { onEvent, events } from '@/lib/tauri'
import { useTimerStore } from '@/features/timer/model/useTimerStore'
import { useTaskStore } from '@/features/tasks/model/useTaskStore'
import type { UnlistenFn } from '@tauri-apps/api/event'

/**
 * Subscribes to all Tauri backend events and reconciles local store state.
 * Real-time countdowns apply the tick payload locally; everything else
 * re-fetches authoritative state from the backend.
 */
export function useEventBus(): void {
  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const applyTick = useTimerStore((s) => s.applyTick)
  const loadTasks = useTaskStore((s) => s.loadTasks)

  useEffect(() => {
    const reload = (): void => {
      void fetchTimer()
      void loadTasks()
    }

    const unlisteners: Array<Promise<UnlistenFn>> = [
      onEvent(events.appInitialized, reload),
      onEvent(events.taskListUpdated, () => void loadTasks()),
      onEvent(events.taskActiveChanged, () => void fetchTimer()),
      onEvent(events.taskCompleted, () => void loadTasks()),
      onEvent(events.taskProgressUpdated, () => void loadTasks()),
      onEvent(events.taskAutoAdvanced, () => {
        void fetchTimer()
        void loadTasks()
        toast.success('Switched to next incomplete task')
      }),
      onEvent(events.timerTick, (payload) => applyTick(payload)),
      onEvent(events.timerStatusChanged, () => void fetchTimer()),
      onEvent(events.timerPhaseCompleted, reload),
      onEvent(events.timerPhaseSkipped, reload),
      onEvent(events.timerReset, () => void fetchTimer()),
      onEvent(events.timerPaused, () => void fetchTimer()),
    ]

    return () => {
      for (const p of unlisteners) void p.then((fn) => fn())
    }
  }, [fetchTimer, applyTick, loadTasks])
}
