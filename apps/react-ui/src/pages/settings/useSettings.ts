import { create } from 'zustand'
import { invokeCmd } from '@/lib/tauri'
import { BackendError } from '@/lib/errors'
import { logger } from '@/lib/logger'
import type { TimerConfiguration } from '@/pages/timer/useTimer'

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

interface SettingsStore {
  config: Config | null
  isLoading: boolean
  error: BackendError | null
  loadConfig: () => Promise<boolean>
  saveConfig: (config: Config) => Promise<boolean>
  resetToDefaults: () => Promise<boolean>
  testAudioPreview: (soundType: string) => Promise<boolean>
  openDataDirectory: () => Promise<boolean>
  clearAllData: () => Promise<boolean>
  applyTheme: (theme: Theme) => void
  clearError: () => void
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  config: null,
  isLoading: false,
  error: null,

  loadConfig: async () => {
    set({ isLoading: true })
    try {
      const config = await invokeCmd('get_global_config')
      set({ config, isLoading: false, error: null })
      get().applyTheme(config.appearance.theme)
      return true
    } catch (e) {
      logger.error('loadConfig failed', e)
      set({ error: e as BackendError, isLoading: false })
      return false
    }
  },

  saveConfig: async (config) => {
    try {
      await invokeCmd('save_global_config', { config })
      set({ config, error: null })
      get().applyTheme(config.appearance.theme)
      return true
    } catch (e) {
      logger.error('saveConfig failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  resetToDefaults: async () => {
    try {
      await invokeCmd('reset_config_to_defaults')
      return await get().loadConfig()
    } catch (e) {
      logger.error('resetToDefaults failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  testAudioPreview: async (soundType) => {
    try {
      await invokeCmd('test_audio_preview', { sound_type: soundType })
      return true
    } catch (e) {
      logger.error('testAudioPreview failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  openDataDirectory: async () => {
    try {
      await invokeCmd('open_data_directory')
      return true
    } catch (e) {
      logger.error('openDataDirectory failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  clearAllData: async () => {
    try {
      await invokeCmd('clear_all_data')
      return true
    } catch (e) {
      logger.error('clearAllData failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  applyTheme: (theme) => {
    const root = document.documentElement
    root.classList.remove('light', 'dark')
    if (theme === Theme.System) {
      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light'
      root.classList.add(systemTheme)
    } else {
      root.classList.add(theme.toLowerCase())
    }
  },

  clearError: () => set({ error: null }),
}))
