import { Trash2, CheckSquare, Square, Crosshair, RotateCcw, Pencil } from 'lucide-react'
import { TaskStatus } from '@/pages/tasks/useTasks'
import type { Task } from '@/pages/tasks/useTasks'

interface TaskRowProps {
  task: Task
  onComplete: () => void
  onReset: () => void
  onDelete: () => void
  onSetActive: () => void
  onEdit: () => void
  timerRunning: boolean
  onNavigateToTimer: () => void
}

export function TaskRow({
  task,
  onComplete,
  onReset,
  onDelete,
  onSetActive,
  onEdit,
  timerRunning,
}: TaskRowProps) {
  const isCompleted = task.status === TaskStatus.Completed
  const isActive = task.status === TaskStatus.Active
  const isRunning = isActive && timerRunning
  const progressPct =
    task.max_sessions > 0
      ? Math.round((task.current_sessions / task.max_sessions) * 100)
      : 0

  return (
    <li
      className={[
        'flex flex-col gap-2 rounded-xl border px-4 py-3.5 transition-colors',
        isActive
          ? 'border-toro/40 bg-toro/10'
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
            <CheckSquare size={20} className="text-toro" />
          ) : (
            <Square size={20} />
          )}
        </button>

        {/* Title + meta */}
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1.5">
            {isActive && (
              <span className="inline-flex shrink-0 items-center gap-1 rounded-full bg-primary px-1.5 py-0.5 text-[9px] font-bold tracking-wide text-primary-foreground uppercase">
                <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-white" />
                Active
              </span>
            )}
            <span
              className={[
                'block truncate text-sm font-medium',
                isCompleted ? 'text-muted-foreground line-through' : '',
              ].join(' ')}
            >
              {task.name}
            </span>
          </div>
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
          {task.max_sessions > 1 && (
            <div className="bg-muted mt-2 h-1.5 w-full overflow-hidden rounded-full">
              <div
                className="h-full rounded-full bg-toro transition-all duration-300"
                style={{ width: `${progressPct}%` }}
              />
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
            title={timerRunning ? 'Go to running timer' : 'Focus on this task'}
            className={[
              'shrink-0 rounded p-1 transition-colors',
              isActive
                ? 'text-toro'
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
          disabled={isRunning}
          title={isRunning ? 'Stop the timer to edit' : 'Edit task'}
          className={[
            'shrink-0 p-1 transition-colors',
            isRunning
              ? 'text-muted-foreground/40 cursor-not-allowed'
              : 'text-muted-foreground hover:text-foreground',
          ].join(' ')}
        >
          <Pencil size={15} />
        </button>

        {/* Delete */}
        <button
          onClick={onDelete}
          disabled={isRunning}
          title={isRunning ? 'Stop the timer to delete' : 'Delete'}
          className={[
            'shrink-0 p-1 transition-colors',
            isRunning
              ? 'text-muted-foreground/40 cursor-not-allowed'
              : 'text-muted-foreground hover:text-destructive',
          ].join(' ')}
        >
          <Trash2 size={15} />
        </button>
      </div>
    </li>
  )
}
