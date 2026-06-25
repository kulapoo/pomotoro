import { useState, useMemo } from 'react'
import { Toaster } from 'sonner'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { ScreenBlocker } from '@/components/ScreenBlocker'
import { Sidebar } from '@/components/layout/Sidebar'
import { Bootstrap } from '@/app/Bootstrap'
import { ErrorWatcher } from '@/app/ErrorWatcher'
import { DevPanel } from '@/app/DevPanel'
import { useEventBus } from '@/app/EventBus'
import { useTaskCycling } from '@/app/keyboard'
import { TimerPage } from '@/pages/timer/TimerPage'
import { TasksPage } from '@/pages/tasks/TasksPage'
import { SettingsPage } from '@/pages/settings/SettingsPage'
import { useSettingsStore } from '@/pages/settings/useSettings'
import type { Page } from '@/app/types'

export function App() {
  const [page, setPage] = useState<Page>('timer')

  // Wire global event subscriptions and keyboard shortcuts.
  useEventBus()
  useTaskCycling()

  const notifConfig = useSettingsStore((s) => s.config?.notification)

  const toasterPosition = useMemo(() => {
    switch (notifConfig?.notification_position) {
      case 'TopLeft':
        return 'top-left' as const
      case 'BottomRight':
        return 'bottom-right' as const
      case 'BottomLeft':
        return 'bottom-left' as const
      case 'Center':
        return 'top-center' as const
      default:
        return 'top-right' as const
    }
  }, [notifConfig?.notification_position])

  return (
    <Bootstrap>
      <ErrorWatcher />
      {import.meta.env.DEV && <DevPanel />}
      <div className="text-foreground flex h-screen w-full overflow-hidden bg-linear-to-br from-indigo-50 via-white to-purple-50 font-sans transition-colors duration-300 dark:from-gray-950 dark:via-gray-900 dark:to-indigo-950">
        <Toaster position={toasterPosition} richColors />
        <Sidebar currentPage={page} onNavigate={setPage} />
        <main className="flex-1 overflow-y-auto p-6 md:p-10">
          {page === 'timer' && (
            <ErrorBoundary>
              <TimerPage onNavigate={setPage} />
            </ErrorBoundary>
          )}
          {page === 'tasks' && (
            <ErrorBoundary>
              <TasksPage onNavigate={setPage} />
            </ErrorBoundary>
          )}
          {page === 'settings' && (
            <ErrorBoundary>
              <SettingsPage />
            </ErrorBoundary>
          )}
        </main>
      </div>
      <ScreenBlocker />
    </Bootstrap>
  )
}
