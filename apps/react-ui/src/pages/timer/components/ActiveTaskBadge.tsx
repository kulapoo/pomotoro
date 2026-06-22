import type { Task } from '@/pages/tasks/useTasks'

interface ActiveTaskBadgeProps {
  task: Task
  running: boolean
}

export function ActiveTaskBadge({ task, running }: ActiveTaskBadgeProps) {
  return (
    <div className="bg-card border-border flex max-w-xs items-center gap-2.5 truncate rounded-full border px-4 py-2 shadow-sm">
      {running && (
        <span className="h-2 w-2 shrink-0 animate-pulse rounded-full bg-indigo-500" />
      )}
      <span className="truncate text-sm font-medium">{task.name}</span>
      <span className="text-muted-foreground shrink-0 text-xs tabular-nums">
        {task.current_sessions}/{task.max_sessions}
      </span>
    </div>
  )
}
