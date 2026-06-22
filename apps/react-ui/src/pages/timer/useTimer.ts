import { create } from 'zustand'
import { invokeCmd } from '@/lib/tauri'
import { BackendError } from '@/lib/errors'
import { logger } from '@/lib/logger'

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
  remaining_seconds?: number
  paused_from?: TimerStateData
}

export interface ActiveTimer {
  task_id: string
  state: TimerStateData
}

export interface Timer {
  task_id: string | null
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
  return timer.state.remaining_seconds ?? 0
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
      const from = timer.state.paused_from
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

export interface TickPayload {
  task_id: string
  phase: Phase
  remaining_seconds: number
  version: number
  occurred_at: string
  config: TimerConfiguration
}

export interface TimerStatusChangedPayload {
  task_id: string
  old_status: string
  new_status: string
  phase: Phase
  version: number
  occurred_at: string
}

export interface PhaseSkippedPayload {
  task_id: string
  skipped_phase: Phase
  next_phase: Phase
  version: number
  occurred_at: string
}

type TaskCommand =
  | 'start_timer'
  | 'pause_timer'
  | 'resume_timer'
  | 'reset_timer'
  | 'reset_timer_phase'
  | 'skip_phase'

interface TimerStore {
  timer: Timer | null
  error: BackendError | null
  fetchTimer: () => Promise<boolean>
  applyTick: (payload: TickPayload) => void
  start: () => Promise<boolean>
  pause: () => Promise<boolean>
  resume: () => Promise<boolean>
  resetTimer: () => Promise<boolean>
  resetPhase: () => Promise<boolean>
  skip: () => Promise<boolean>
  clearError: () => void
}

export const useTimerStore = create<TimerStore>((set, get) => ({
  timer: null,
  error: null,

  fetchTimer: async () => {
    try {
      const timer = await invokeCmd('get_timer_state')
      set({ timer, error: null })
      return true
    } catch (e) {
      logger.error('fetchTimer failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  applyTick: (payload) => {
    const timer = get().timer
    if (!timer) return
    const state = timer.state.state
    if (state === TimerState.Idle || state === TimerState.Paused) return

    if (payload.task_id !== timer.task_id) return
    const phaseByState: Partial<Record<TimerStateName, Phase>> = {
      [TimerState.Working]: Phase.Work,
      [TimerState.ShortBreak]: Phase.ShortBreak,
      [TimerState.LongBreak]: Phase.LongBreak,
    }
    if (phaseByState[state] !== payload.phase) return

    set({
      timer: {
        ...timer,
        state: {
          ...timer.state,
          remaining_seconds: payload.remaining_seconds,
        },
      },
    })
  },

  start: async () => runWithTask(set, get, 'start_timer'),
  pause: async () => runWithTask(set, get, 'pause_timer'),
  resume: async () => runWithTask(set, get, 'resume_timer'),
  resetTimer: async () => runWithTask(set, get, 'reset_timer'),
  resetPhase: async () => runWithTask(set, get, 'reset_timer_phase'),
  skip: async () => runWithTask(set, get, 'skip_phase'),

  clearError: () => set({ error: null }),
}))

async function runWithTask(
  set: (partial: Partial<TimerStore>) => void,
  get: () => TimerStore,
  command: TaskCommand,
): Promise<boolean> {
  const taskId = get().timer?.task_id
  if (!taskId) return false
  try {
    await invokeCmd(command, { task_id: taskId })
    return true
  } catch (e) {
    logger.error(`${command} failed`, e)
    set({ error: e as BackendError })
    return false
  }
}
