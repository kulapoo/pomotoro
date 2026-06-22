import { useTaskStore } from '@/pages/tasks/useTasks'
import { DEFAULT_DURATIONS } from '@/lib/duration'

export function SessionDots() {
  const activeTask = useTaskStore((s) => s.activeTask)

  const cycleLen =
    activeTask?.config?.timer?.sessions_until_long_break ??
    DEFAULT_DURATIONS.sessionsUntilLongBreak
  const hasFixedSessions = (activeTask?.max_sessions ?? 0) > 0
  const dotTotal = activeTask
    ? Math.max(0, hasFixedSessions ? activeTask.max_sessions : cycleLen)
    : 0
  const dotFilled = activeTask
    ? hasFixedSessions
      ? Math.min(activeTask.current_sessions, activeTask.max_sessions)
      : activeTask.current_sessions % cycleLen
    : 0

  if (!activeTask || dotTotal === 0) return null

  const dots = Array.from({ length: dotTotal }, (_, i) => i)

  return (
    <div className="flex items-center gap-2">
      {dots.map((i) => (
        <div
          key={i}
          className={[
            'h-2.5 w-2.5 rounded-full transition-all duration-300',
            i < dotFilled ? 'bg-indigo-500' : 'bg-muted-foreground/25',
          ].join(' ')}
        />
      ))}
    </div>
  )
}
