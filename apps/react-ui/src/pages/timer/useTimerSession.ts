import { useShallow } from 'zustand/react/shallow'
import {
  useTimerStore,
  Phase,
  getRemainingSeconds,
  getEffectivePhase,
  isTimerRunning,
  isTimerPaused,
  isTimerIdle,
} from '@/pages/timer/useTimer'
import { useTaskStore, TaskStatus } from '@/pages/tasks/useTasks'
import { phaseDuration, DEFAULT_DURATIONS } from '@/lib/duration'

/**
 * Transition-stable session state. Every selector returns a primitive
 * (Phase string or boolean) or reads from the task store, so consumers do
 * NOT re-render on the 1-second timer tick.
 */
export function useTimerSession() {
  const phase = useTimerStore((s) => {
    const t = s.timer
    return t ? getEffectivePhase(t) : Phase.Work
  })
  const running = useTimerStore((s) => (s.timer ? isTimerRunning(s.timer) : false))
  const paused = useTimerStore((s) => (s.timer ? isTimerPaused(s.timer) : false))
  const idle = useTimerStore((s) => (s.timer ? isTimerIdle(s.timer) : false))
  const hasTaskId = useTimerStore((s) => !!s.timer?.task_id)
  const activeTask = useTaskStore((s) => s.activeTask)

  const timerCfg = activeTask?.config?.timer ?? null
  const isTaskCompleted = activeTask?.status === TaskStatus.Completed
  const isBreakPhase = phase === Phase.ShortBreak || phase === Phase.LongBreak
  const isLastBreak = !!isTaskCompleted && isBreakPhase
  const canStart = hasTaskId && !isTaskCompleted
  const canPlayPause = !!activeTask && (canStart || running || paused)

  return {
    activeTask,
    timerCfg,
    phase,
    running,
    paused,
    idle,
    isTaskCompleted,
    isLastBreak,
    canStart,
    canPlayPause,
  }
}

/**
 * Tick-sensitive countdown values. Only TimerRing should use this — it
 * intentionally re-renders every second as `remaining` counts down.
 */
export function useTimerCountdown() {
  const timerCfg = useTaskStore((s) => s.activeTask?.config?.timer ?? null)
  return useTimerStore(
    useShallow((s) => {
      const t = s.timer
      const idle = t ? isTimerIdle(t) : false
      const rawRemaining = t ? getRemainingSeconds(t) : 0
      const idleDuration = timerCfg?.work_duration ?? DEFAULT_DURATIONS.work
      const remaining = idle ? idleDuration : rawRemaining
      const phase = t ? getEffectivePhase(t) : Phase.Work
      const total = phaseDuration(phase, timerCfg)
      return { remaining, total, phase }
    }),
  )
}
