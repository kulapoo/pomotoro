import { useShallow } from 'zustand/react/shallow'
import {
  useTimerStore,
  Phase,
  getRemainingSeconds,
  getEffectivePhase,
  isTimerIdle,
} from '@/pages/timer/useTimer'
import { useTaskStore } from '@/pages/tasks/useTasks'
import { phaseDuration, DEFAULT_DURATIONS } from '@/lib/duration'

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
