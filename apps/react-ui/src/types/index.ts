export type Page = 'timer' | 'tasks' | 'settings'

// --- Theme ---

export const Theme = {
  Light: 'Light',
  Dark: 'Dark',
  System: 'System',
} as const
export type Theme = typeof Theme[keyof typeof Theme]

export type TaskCyclingBehavior = 'Manual' | 'AutoAdvance'
export type NotificationPosition =
  | 'TopRight'
  | 'TopLeft'
  | 'BottomRight'
  | 'BottomLeft'
  | 'Center'

// --- Timer ---

export const TimerState = {
  Idle: 'Idle',
  Working: 'Working',
  ShortBreak: 'ShortBreak',
  LongBreak: 'LongBreak',
  Paused: 'Paused',
} as const
export type TimerStateName = typeof TimerState[keyof typeof TimerState]

export const Phase = {
  Work: 'Work',
  ShortBreak: 'ShortBreak',
  LongBreak: 'LongBreak',
} as const
export type Phase = typeof Phase[keyof typeof Phase]

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
  return s === TimerState.Working || s === TimerState.ShortBreak || s === TimerState.LongBreak
}

export function isTimerPaused(timer: Timer): boolean {
  return timer.state.state === TimerState.Paused
}

export function isTimerIdle(timer: Timer): boolean {
  return timer.state.state === TimerState.Idle
}

// --- Task ---

export const TaskStatus = {
  Active: 'Active',
  Queued: 'Queued',
  Completed: 'Completed',
  Paused: 'Paused',
} as const
export type TaskStatus = typeof TaskStatus[keyof typeof TaskStatus]

export interface TimerConfiguration {
  work_duration: number
  short_break_duration: number
  long_break_duration: number
  sessions_until_long_break: number
}

export interface AppearanceConfig {
  theme: Theme
  show_seconds_in_display: boolean
  always_on_top: boolean
  compact_mode: boolean
  show_task_list_sidebar: boolean
  animate_progress: boolean
}

export interface NotificationConfig {
  enable_desktop_notifications: boolean
  enable_sound_notifications: boolean
  show_phase_transition_notifications: boolean
  show_task_completion_notifications: boolean
  notification_position: NotificationPosition
  auto_dismiss_delay_seconds: number
}

export interface GeneralConfig {
  task_cycling_behavior: TaskCyclingBehavior
  auto_start_breaks: boolean
  auto_start_work_after_break: boolean
  minimize_to_tray: boolean
  start_minimized: boolean
  enable_screen_blocking: boolean
  block_screen_after_work: boolean
  block_screen_after_work_message: string
  block_screen_after_break: boolean
  block_screen_after_break_message: string
  persistence_interval_seconds: number
}

export interface AudioConfig {
  muted: boolean
  volume: number
  enable_background_audio: boolean
  work_notification_sound?: string | null
  break_notification_sound?: string | null
  background_sound?: string | null
}

export interface Config {
  timer: TimerConfiguration
  audio: AudioConfig
  general: GeneralConfig
  notification: NotificationConfig
  appearance: AppearanceConfig
}

export interface Task {
  id: string
  name: string
  description: string | null
  max_sessions: number
  current_sessions: number
  tags: string[]
  config: Config
  created_at: string
  updated_at: string
  completed_at: string | null
  status: TaskStatus
  default: boolean
}

// --- Create/Update request shapes ---

export interface CreateTaskRequest {
  name: string
  description?: string
  max_sessions: number
  tags: string[]
  work_duration?: number
  short_break_duration?: number
  long_break_duration?: number
  sessions_until_long_break?: number
}

export interface UpdateTaskRequest {
  id: string
  name?: string
  description?: string
  max_sessions?: number
  tags?: string[]
  work_duration?: number
  short_break_duration?: number
  long_break_duration?: number
  sessions_until_long_break?: number
}

// --- App events (must match ui_listeners in the Rust backend) ---

export const AppEvents = {
  AppInitialized: 'app:initialized',
  TaskListUpdated: 'task:list_updated',
  TaskActiveChanged: 'task:active_changed',
  TaskCompleted: 'task:task_completed',
  TaskProgressUpdated: 'task:progress_updated',
  TaskAutoAdvanced: 'task:auto_advanced',
  TimerTick: 'timer:tick',
  TimerStatusChanged: 'timer:status_changed',
  TimerPhaseCompleted: 'timer:phase_completed',
  TimerPhaseSkipped: 'timer:phase_skipped',
  TimerReset: 'timer:timer_reset',
  TimerPaused: 'timer:timer_paused',
  TimerResumed: 'timer:timer_resumed',
  ScreenBlockerActivate: 'screen_blocker:activate',
} as const
