import { Trash2, CheckCircle2, Circle, Crosshair, RotateCcw, Pencil } from 'lucide-react'
import { TaskStatus } from '@/features/tasks/types'
import type { Task } from '@/features/tasks/types'

interface TaskRowProps {
  task: Task
  onComplete: () => void
  onReset: () => void
  onDelete: () => void
  onSetActive: () => void
  onEdit: () => void
}

export function TaskRow({
  task,
  onComplete,
  onReset,
  onDelete,
  onSetActive,
  onEdit,
}: TaskRowProps) {
  const isCompleted = task.status === TaskStatus.Completed
  const isActive = task.status === TaskStatus.Active
  const progressPct =
    task.max_sessions > 0
      ? Math.round((task.current_sessions / task.max_sessions) * 100)
      : 0

  return (
    <li
      className={[
        'flex flex-col gap-2 rounded-xl border px-4 py-3.5 transition-colors',
        isActive
          ? 'border-indigo-400/60 bg-indigo-50/60 dark:bg-indigo-950/20'
          : 'border-border bg-card',
        isCompleted ? 'opacity-55' : '',
      ].join(' ')}
    >
      <div className="flex items-center gap-3">
        {/* Complete toggle */}
        <button
          onClick={isCompleted ? onReset : onComplete}
          className="text-muted-foreground hover:text-foreground shrink-0 transition-colors"
          title={isCompleted ? 'Reopen' : 'Complete'}
        >
          {isCompleted ? (
            <CheckCircle2 size={20} className="text-indigo-500" />
          ) : (
            <Circle size={20} />
          )}
        </button>

        {/* Title + meta */}
        <div className="min-w-0 flex-1">
          <span
            className={[
              'block truncate text-sm font-medium',
              isCompleted ? 'text-muted-foreground line-through' : '',
            ].join(' ')}
          >
            {task.name}
          </span>
          {task.description && (
            <p className="text-muted-foreground mt-0.5 truncate text-xs">
              {task.description}
            </p>
          )}
          {task.tags.length > 0 && (
            <div className="mt-1 flex flex-wrap gap-1">
              {task.tags.map((tag) => (
                <span
                  key={tag}
                  className="bg-muted text-muted-foreground rounded-full px-1.5 py-0.5 text-[10px]"
                >
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>

        {/* Session count */}
        <span className="text-muted-foreground shrink-0 text-xs tabular-nums">
          {task.current_sessions}/{task.max_sessions}
        </span>

        {/* Focus */}
        {!isCompleted && (
          <button
            onClick={onSetActive}
            title="Focus on this task"
            className={[
              'shrink-0 rounded p-1 transition-colors',
              isActive
                ? 'text-indigo-500'
                : 'text-muted-foreground hover:text-foreground',
            ].join(' ')}
          >
            <Crosshair size={15} />
          </button>
        )}

        {/* Reset progress */}
        {task.current_sessions > 0 && !isCompleted && (
          <button
            onClick={onReset}
            title="Reset progress"
            className="text-muted-foreground hover:text-foreground shrink-0 p-1 transition-colors"
          >
            <RotateCcw size={15} />
          </button>
        )}

        {/* Edit */}
        <button
          onClick={onEdit}
          title="Edit task"
          className="text-muted-foreground hover:text-foreground shrink-0 p-1 transition-colors"
        >
          <Pencil size={15} />
        </button>

        {/* Delete */}
        <button
          onClick={onDelete}
          title="Delete"
          className="text-muted-foreground hover:text-destructive shrink-0 p-1 transition-colors"
        >
          <Trash2 size={15} />
        </button>
      </div>

      {/* Progress bar */}
      {task.max_sessions > 1 && (
        <div className="bg-muted ml-8 h-1.5 w-full overflow-hidden rounded-full">
          <div
            className="h-full rounded-full bg-indigo-500 transition-all duration-300"
            style={{ width: `${progressPct}%` }}
          />
        </div>
      )}
    </li>
  )
}
