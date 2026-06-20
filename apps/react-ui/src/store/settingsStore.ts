import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { Theme } from '@/types'
import type { Config } from '@/types'

interface SettingsStore {
  config: Config | null
  isLoading: boolean
  error: string | null

  loadConfig: () => Promise<void>
  saveConfig: (config: Config) => Promise<void>
  resetToDefaults: () => Promise<void>
  applyTheme: (theme: Theme) => void
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  config: null,
  isLoading: false,
  error: null,

  loadConfig: async () => {
    set({ isLoading: true })
    try {
      const config = await invoke<Config>('get_global_config')
      set({ config, isLoading: false, error: null })
      get().applyTheme(config.appearance.theme)
    } catch (e) {
      set({ error: String(e), isLoading: false })
    }
  },

  saveConfig: async (config) => {
    try {
      await invoke('save_global_config', { config })
      set({ config })
      get().applyTheme(config.appearance.theme)
    } catch (e) {
      set({ error: String(e) })
    }
  },

  resetToDefaults: async () => {
    try {
      await invoke('reset_config_to_defaults')
      await get().loadConfig()
    } catch (e) {
      set({ error: String(e) })
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
}))
