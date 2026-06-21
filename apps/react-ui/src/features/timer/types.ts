export const TimerState = {
  Idle: 'Idle',
  Working: 'Working',
  ShortBreak: 'ShortBreak',
  LongBreak: 'LongBreak',
  Paused: 'Paused',
} as const
export type TimerStateName = (typeof TimerState)[keyof typeof TimerState]

export const Phase = {
  Work: 'Work',
  ShortBreak: 'ShortBreak',
  LongBreak: 'LongBreak',
} as const
export type Phase = (typeof Phase)[keyof typeof Phase]

export interface TimerStateData {
  state: TimerStateName
  data?: {
    remaining_seconds: number
    paused_from?: TimerStateData
  }
}

export interface Timer {
  task_id: string
  state: TimerStateData
}

export interface TimerConfiguration {
  work_duration: number
  short_break_duration: number
  long_break_duration: number
  sessions_until_long_break: number
}

export function getRemainingSeconds(timer: Timer): number {
  if (timer.state.state === TimerState.Idle) return 0
  return timer.state.data?.remaining_seconds ?? 0
}

export function getEffectivePhase(timer: Timer): Phase {
  switch (timer.state.state) {
    case TimerState.Working:
      return Phase.Work
    case TimerState.ShortBreak:
      return Phase.ShortBreak
    case TimerState.LongBreak:
      return Phase.LongBreak
    case TimerState.Paused: {
      const from = timer.state.data?.paused_from
      if (from?.state === TimerState.ShortBreak) return Phase.ShortBreak
      if (from?.state === TimerState.LongBreak) return Phase.LongBreak
      return Phase.Work
    }
    default:
      return Phase.Work
  }
}

export function isTimerRunning(timer: Timer): boolean {
  const s = timer.state.state
  return (
    s === TimerState.Working || s === TimerState.ShortBreak || s === TimerState.LongBreak
  )
}

export function isTimerPaused(timer: Timer): boolean {
  return timer.state.state === TimerState.Paused
}

export function isTimerIdle(timer: Timer): boolean {
  return timer.state.state === TimerState.Idle
}
