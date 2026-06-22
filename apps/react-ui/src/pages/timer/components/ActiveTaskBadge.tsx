import { useTaskStore } from '@/pages/tasks/useTasks'
import { useTimerSession } from '@/pages/timer/useTimerSession'

export function ActiveTaskBadge() {
  const activeTask = useTaskStore((s) => s.activeTask)
  const { running } = useTimerSession()

  if (!activeTask) return null

  return (
    <div className="bg-card border-border flex max-w-xs items-center gap-2.5 truncate rounded-full border px-4 py-2 shadow-sm">
      {running && (
        <span className="h-2 w-2 shrink-0 animate-pulse rounded-full bg-indigo-500" />
      )}
      <span className="truncate text-sm font-medium">{activeTask.name}</span>
      <span className="text-muted-foreground shrink-0 text-xs tabular-nums">
        {activeTask.current_sessions}/{activeTask.max_sessions}
      </span>
    </div>
  )
}
