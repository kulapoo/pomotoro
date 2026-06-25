import type { CSSProperties } from 'react'

interface ToroIconProps {
  size?: number
  className?: string
  style?: CSSProperties
}

/**
 * Brand mark: a front-view bull-head (toro) silhouette.
 * Filled with `currentColor` so it inherits the surrounding text color.
 */
export function ToroIcon({ size = 24, className, style }: ToroIconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      className={className}
      style={style}
      aria-hidden="true"
    >
      <path
        fill="currentColor"
        d="M3 3.5C4 6 6.5 7.2 9 7c1-.5 2-1 3-1s2 .5 3 1c2.5.2 5-1 6-3.5-.5 4.5-2 6.5-4 7.5-.5 3-1 6-2 8-1 1.5-2 2-3 2s-2-.5-3-2c-1-2-1.5-5-2-8-2-1-3.5-3-4-7.5Z"
      />
    </svg>
  )
}
