import { Phase } from '@/pages/timer/useTimer'
import { useTimerCountdown } from '@/pages/timer/useTimerCountdown'
import { formatClock } from '@/lib/duration'
import { ToroIcon } from '@/components/ui/ToroIcon'

const RING_R = 90
const CIRC = 2 * Math.PI * RING_R

const PHASE_ARC_COLOR: Record<Phase, string> = {
  [Phase.Work]: 'var(--toro)',
  [Phase.ShortBreak]: 'var(--break-short)',
  [Phase.LongBreak]: 'var(--break-long)',
}

export function TimerRing() {
  const { phase, remaining, total } = useTimerCountdown()
  const progress = total > 0 ? Math.min(1, Math.max(0, remaining / total)) : 1
  const arcOffset = CIRC * (1 - progress)

  return (
    <div
      className="relative flex items-center justify-center"
      style={{
        width: 'min(248px, 65vw)',
        height: 'min(248px, 65vw)',
      }}
    >
      <svg className="h-full w-full -rotate-90" viewBox="0 0 200 200">
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
      <div className="absolute inset-0 flex flex-col items-center justify-center gap-1.5">
        <ToroIcon
          size={0}
          style={{ color: PHASE_ARC_COLOR[phase] }}
          className="h-[min(26px,7vw)] w-[min(26px,7vw)]"
        />
        <span
          className="font-mono font-bold tracking-tight tabular-nums select-none"
          style={{ fontSize: 'min(3.5rem, 16vw)' }}
        >
          {formatClock(remaining)}
        </span>
      </div>
    </div>
  )
}
