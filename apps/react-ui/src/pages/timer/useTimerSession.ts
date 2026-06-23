import {
  useTimerStore,
  Phase,
  getEffectivePhase,
  isTimerRunning,
  isTimerPaused,
  isTimerIdle,
} from '@/pages/timer/useTimer'
import { useTaskStore, TaskStatus } from '@/pages/tasks/useTasks'

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
  const allPhasesCompleted =
    isTaskCompleted && activeTask.current_sessions === activeTask.max_sessions && !running

  const canPlayPause =
    !!activeTask && !allPhasesCompleted && (canStart || running || paused)

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
    allPhasesCompleted,
  }
}
