import { useEffect } from 'react'
import { toastError } from '@/lib/toast'
import { useTimerStore } from '@/features/timer/model/useTimerStore'
import { useTaskStore } from '@/features/tasks/model/useTaskStore'
import { useSettingsStore } from '@/features/settings/model/useSettingsStore'

/**
 * Render-less component that watches every store's `error` field and surfaces
 * it through the unified toast pipeline, then clears it. This is the single
 * source of error toasts — components no longer catch/store/toast themselves.
 */
export function ErrorWatcher() {
  const timerError = useTimerStore((s) => s.error)
  const taskError = useTaskStore((s) => s.error)
  const settingsError = useSettingsStore((s) => s.error)
  const clearTimerError = useTimerStore((s) => s.clearError)
  const clearTaskError = useTaskStore((s) => s.clearError)
  const clearSettingsError = useSettingsStore((s) => s.clearError)

  useEffect(() => {
    if (!timerError) return
    toastError(timerError)
    clearTimerError()
  }, [timerError, clearTimerError])

  useEffect(() => {
    if (!taskError) return
    toastError(taskError)
    clearTaskError()
  }, [taskError, clearTaskError])

  useEffect(() => {
    if (!settingsError) return
    toastError(settingsError)
    clearSettingsError()
  }, [settingsError, clearSettingsError])

  return null
}
