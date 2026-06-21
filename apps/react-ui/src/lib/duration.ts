import { Phase } from '@/features/timer/types'
import type { TimerConfiguration } from '@/features/timer/types'

/**
 * Default pomodoro durations in seconds. Single source of truth — replaces
 * the previously hardcoded 1500/300/900/4 scattered across the UI.
 */
export const DEFAULT_DURATIONS = {
  work: 1500,
  shortBreak: 300,
  longBreak: 900,
  sessionsUntilLongBreak: 4,
} as const

export type DurationUnit = 'seconds' | 'minutes'

/** Convert a UI-entered value to seconds based on the chosen unit. */
export function toSeconds(value: number, unit: DurationUnit): number {
  return unit === 'seconds' ? value : value * 60
}

/** Convert a stored seconds value to the UI display unit. */
export function fromSeconds(value: number, unit: DurationUnit): number {
  return unit === 'seconds' ? value : Math.round(value / 60)
}

/** Format seconds as `MM:SS`. */
export function formatClock(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

/** Resolve the total duration (seconds) for a phase, falling back to defaults. */
export function phaseDuration(
  phase: Phase,
  config: TimerConfiguration | null | undefined,
): number {
  switch (phase) {
    case Phase.ShortBreak:
      return config?.short_break_duration ?? DEFAULT_DURATIONS.shortBreak
    case Phase.LongBreak:
      return config?.long_break_duration ?? DEFAULT_DURATIONS.longBreak
    default:
      return config?.work_duration ?? DEFAULT_DURATIONS.work
  }
}
