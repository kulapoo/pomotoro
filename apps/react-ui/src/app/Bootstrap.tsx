import { useCallback, useEffect, useState } from 'react'
import type { ReactNode } from 'react'
import { onEvent, events } from '@/lib/tauri'
import { useTimerStore } from '@/pages/timer/useTimer'
import { useTaskStore } from '@/pages/tasks/useTasks'
import { useSettingsStore } from '@/pages/settings/useSettings'

interface BootstrapProps {
  children: ReactNode
}

/**
 * Performs the initial data load and gates the app behind a splash screen.
 * Unlike the previous `Promise.allSettled` swallow, failures are surfaced
 * with a Retry button so a half-loaded state is never silently accepted.
 */
export function Bootstrap({ children }: BootstrapProps) {
  const [phase, setPhase] = useState<'loading' | 'failed' | 'ready'>('loading')

  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const loadTasks = useTaskStore((s) => s.loadTasks)
  const loadConfig = useSettingsStore((s) => s.loadConfig)

  const init = useCallback(async () => {
    setPhase('loading')
    const [timerOk, tasksOk, configOk] = await Promise.all([
      fetchTimer(),
      loadTasks(),
      loadConfig(),
    ])
    setPhase(timerOk && tasksOk && configOk ? 'ready' : 'failed')
  }, [fetchTimer, loadTasks, loadConfig])

  useEffect(() => {
    void init()
  }, [init])

  // Re-run whenever the backend signals (re)initialization.
  useEffect(() => {
    const p = onEvent(events.appInitialized, () => void init())
    return () => {
      void p.then((fn) => fn())
    }
  }, [init])

  if (phase === 'ready') return <>{children}</>

  return (
    <div className="flex h-screen w-full flex-col items-center justify-center gap-3 bg-linear-to-br from-indigo-50 via-white to-purple-50 dark:from-gray-950 dark:via-gray-900 dark:to-indigo-950">
      {phase === 'loading' ? (
        <span className="text-muted-foreground animate-pulse text-sm">
          Starting Pomotoro…
        </span>
      ) : (
        <>
          <span className="text-destructive text-sm">
            Some data failed to load. Check your logs and retry.
          </span>
          <button
            onClick={() => void init()}
            className="border-border hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
          >
            Retry
          </button>
        </>
      )}
    </div>
  )
}
