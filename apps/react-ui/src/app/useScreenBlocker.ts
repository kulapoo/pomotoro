import { create } from 'zustand'
import { invokeCmd } from '@/lib/tauri'
import { logger } from '@/lib/logger'

interface ScreenBlockerStore {
  isBlocking: boolean
  message: string
  /** Show the blocker overlay. Called when `screen_blocker:activate` fires. */
  activate: (message: string) => void
  /** User-initiated dismiss: restores the native window then clears state. */
  dismiss: () => Promise<void>
}

export const useScreenBlockerStore = create<ScreenBlockerStore>((set) => ({
  isBlocking: false,
  message: '',

  activate: (message) => {
    set({ isBlocking: true, message })
    invokeCmd('activate_screen_block').catch((e: unknown) => {
      logger.error('activate_screen_block failed', e)
    })
  },

  dismiss: async () => {
    try {
      await invokeCmd('deactivate_screen_block')
    } catch (e) {
      logger.error('deactivate_screen_block failed', e)
    }
    set({ isBlocking: false, message: '' })
  },
}))
