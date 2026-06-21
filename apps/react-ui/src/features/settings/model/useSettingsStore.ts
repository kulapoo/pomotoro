import { create } from 'zustand'
import { invokeCmd } from '@/lib/tauri'
import { BackendError } from '@/lib/errors'
import { logger } from '@/lib/logger'
import { Theme } from '@/features/settings/types'
import type { Config } from '@/features/settings/types'

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
