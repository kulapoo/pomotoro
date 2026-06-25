import { Phase } from '@/pages/timer/useTimer'
import { useTimerCountdown } from '@/pages/timer/useTimerCountdown'
import { formatClock } from '@/lib/duration'
import { ToroIcon } from '@/components/ui/ToroIcon'

const RING_R = 90
const CIRC = 2 * Math.PI * RING_R

const PHASE_ARC_COLOR: Record<Phase, string> = {
  [Phase.Work]: 'var(--toro)',
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
      <div className="absolute flex flex-col items-center gap-1.5">
        <ToroIcon size={26} style={{ color: PHASE_ARC_COLOR[phase] }} />
        <span className="font-mono text-7xl font-bold tracking-tight tabular-nums select-none">
          {formatClock(remaining)}
        </span>
      </div>
    </div>
  )
}
