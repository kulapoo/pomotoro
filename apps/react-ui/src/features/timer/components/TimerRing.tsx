interface TimerRingProps {
  remainingLabel: string
  arcColor: string
  arcOffset: number
}

const RING_R = 90
const CIRC = 2 * Math.PI * RING_R

export function TimerRing({ remainingLabel, arcColor, arcOffset }: TimerRingProps) {
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
            stroke: arcColor,
            strokeDashoffset: arcOffset,
            transition: 'stroke-dashoffset 1s linear, stroke 0.4s ease',
          }}
        />
      </svg>
      <span className="absolute font-mono text-7xl font-bold tracking-tight tabular-nums select-none">
        {remainingLabel}
      </span>
    </div>
  )
}

export { CIRC }
