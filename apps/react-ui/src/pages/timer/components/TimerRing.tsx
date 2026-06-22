import { Phase } from '@/pages/timer/useTimer'
import { useTimerCountdown } from '@/pages/timer/useTimerSession'
import { formatClock } from '@/lib/duration'

const RING_R = 90
const CIRC = 2 * Math.PI * RING_R

const PHASE_ARC_COLOR: Record<Phase, string> = {
  [Phase.Work]: '#6366f1',
  [Phase.ShortBreak]: '#10b981',
  [Phase.LongBreak]: '#3b82f6',
}

export function TimerRing() {
  const { phase, remaining, total } = useTimerCountdown()
  const progress = total > 0 ? Math.min(1, Math.max(0, remaining / total)) : 1
  const arcOffset = CIRC * (1 - progress)

  return (
    <div
      className="relative flex items-center justify-center"
      style={{ width: 248, height: 248 }}
    >
      <svg className="-rotate-90" viewBox="0 0 200 200" width={248} height={248}>
        {/* Background track */}
        <circle
          cx="100"
          cy="100"
          r={RING_R}
          fill="none"
          stroke="currentColor"
          strokeWidth="5"
          className="text-muted-foreground/15"
        />
        {/* Countdown arc */}
        <circle
          cx="100"
          cy="100"
          r={RING_R}
          fill="none"
          strokeWidth="5"
          strokeLinecap="round"
          strokeDasharray={CIRC}
          style={{
            stroke: PHASE_ARC_COLOR[phase],
            strokeDashoffset: arcOffset,
            transition: 'stroke-dashoffset 1s linear, stroke 0.4s ease',
          }}
        />
      </svg>
      <span className="absolute font-mono text-7xl font-bold tracking-tight tabular-nums select-none">
        {formatClock(remaining)}
      </span>
    </div>
  )
}
