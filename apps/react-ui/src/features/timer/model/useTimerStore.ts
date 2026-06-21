import { create } from 'zustand'
import { invokeCmd } from '@/lib/tauri'
import { BackendError } from '@/lib/errors'
import { logger } from '@/lib/logger'
import { TimerState, Phase } from '@/features/timer/types'
import type { Timer, TimerStateName } from '@/features/timer/types'

export interface TickPayload {
  task_id: string
  phase: Phase
  remaining_seconds: number
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

    // Reject stale ticks from the wrong task or from a prior phase.
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
          data: {
            ...timer.state.data,
            remaining_seconds: payload.remaining_seconds,
          },
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
