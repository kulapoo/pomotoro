import { useEffect, useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { onEvent, events } from '@/lib/tauri'
import { createBatchedLoader } from '@/lib/async'
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
  const loadActiveTask = useTaskStore((s) => s.loadActiveTask)

  const init = useRef(
    createBatchedLoader(async () => {
      setPhase('loading')
      const [timerOk, tasksOk, configOk, activeTaskOk] = await Promise.all([
        fetchTimer(),
        loadTasks(),
        loadConfig(),
        loadActiveTask(),
      ])
      setPhase(timerOk && tasksOk && configOk && activeTaskOk ? 'ready' : 'failed')
    }),
  ).current

  useEffect(() => {
    init()
  }, [init])

  // Re-run whenever the backend signals (re)initialization.
  useEffect(() => {
    const p = onEvent(events.appInitialized, () => init())
    return () => {
      void p.then((fn) => fn())
    }
  }, [init])

  if (phase === 'ready') return <>{children}</>

  return (
    <div className="flex h-screen w-full flex-col items-center justify-center gap-3 bg-linear-to-br from-rose-50 via-white to-rose-100 dark:from-gray-950 dark:via-gray-900 dark:to-rose-950">
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
            onClick={() => init()}
            className="border-border hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
          >
            Retry
          </button>
        </>
      )}
    </div>
  )
}
