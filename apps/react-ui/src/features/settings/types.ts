import type { TimerConfiguration } from '@/features/timer/types'

export const Theme = {
  Light: 'Light',
  Dark: 'Dark',
  System: 'System',
} as const
export type Theme = (typeof Theme)[keyof typeof Theme]

export type TaskCyclingBehavior = 'Manual' | 'AutoAdvance'
export type NotificationPosition =
  | 'TopRight'
  | 'TopLeft'
  | 'BottomRight'
  | 'BottomLeft'
  | 'Center'

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
