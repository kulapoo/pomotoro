import { useTimerSession } from '@/pages/timer/useTimerSession'

export function TimerStatus() {
  const { isLastBreak, isTaskCompleted, running, paused } = useTimerSession()

  if (isLastBreak && running) {
    return (
      <span className="text-xs font-medium text-emerald-600 dark:text-emerald-400">
        All sessions complete — this is your final break
      </span>
    )
  }

  return (
    <span className="text-muted-foreground text-xs capitalize">
      {isTaskCompleted
        ? 'Task completed'
        : running
          ? 'Running'
          : paused
            ? 'Paused'
            : 'Ready'}
    </span>
  )
}
