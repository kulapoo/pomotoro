import { useTimerStore } from '@/pages/timer/useTimer'
import { TimerRing } from '@/pages/timer/components/TimerRing'
import { TimerControls } from '@/pages/timer/components/TimerControls'
import { SessionDots } from '@/pages/timer/components/SessionDots'
import { ActiveTaskBadge } from '@/pages/timer/components/ActiveTaskBadge'
import { ActiveTaskActions } from '@/pages/timer/components/ActiveTaskActions'
import { TimerPhaseLabel } from '@/pages/timer/components/TimerPhaseLabel'
import { TimerStatus } from '@/pages/timer/components/TimerStatus'
import { EmptyTaskPrompt } from '@/pages/timer/components/EmptyTaskPrompt'
import type { Page } from '@/app/types'

interface TimerPageProps {
  onNavigate: (page: Page) => void
}

export function TimerPage({ onNavigate }: TimerPageProps) {
  const timer = useTimerStore((s) => s.timer)
  const timerError = useTimerStore((s) => s.error)
  const fetchTimer = useTimerStore((s) => s.fetchTimer)

  if (!timer) {
    return (
      <div className="text-muted-foreground flex h-full flex-col items-center justify-center gap-3">
        {timerError ? (
          <>
            <span className="text-destructive text-sm">{timerError.message}</span>
            <button
              onClick={() => void fetchTimer()}
              className="border-border hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
            >
              Retry
            </button>
          </>
        ) : (
          <span>Initializing timer…</span>
        )}
      </div>
    )
  }

  return (
    <div className="flex min-h-full flex-col items-center justify-center gap-5 py-10">
      <TimerPhaseLabel />
      <TimerRing />
      <SessionDots />
      <ActiveTaskBadge />
      <TimerControls />
      <EmptyTaskPrompt onNavigate={onNavigate} />
      <TimerStatus />
      <ActiveTaskActions />
    </div>
  )
}
