import { Phase } from '@/pages/timer/useTimer'
import { useTimerSession } from '@/pages/timer/useTimerSession'

const PHASE_LABEL: Record<Phase, string> = {
  [Phase.Work]: 'Focus',
  [Phase.ShortBreak]: 'Short Break',
  [Phase.LongBreak]: 'Long Break',
}

const PHASE_COLOR: Record<Phase, string> = {
  [Phase.Work]: 'text-toro',
  [Phase.ShortBreak]: 'text-emerald-500 dark:text-emerald-400',
  [Phase.LongBreak]: 'text-blue-500 dark:text-blue-400',
}

export function TimerPhaseLabel() {
  const { phase } = useTimerSession()
  return (
    <span
      className={`text-xs font-bold tracking-[0.2em] uppercase ${PHASE_COLOR[phase]}`}
    >
      {PHASE_LABEL[phase]}
    </span>
  )
}
