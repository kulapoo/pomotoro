import { memo } from 'react'
import { Trash2, Check, Undo2, Crosshair, RotateCcw, Pencil } from 'lucide-react'
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
  activeTaskId?: string | null
  onNavigateToTimer: () => void
  isSelected: boolean
  onToggleSelect: () => void
}

export const TaskRow = memo(function TaskRow({
  task,
  onComplete,
  onReset,
  onDelete,
  onSetActive,
  onEdit,
  timerRunning,
  activeTaskId,
  isSelected,
  onToggleSelect,
}: TaskRowProps) {
  const isCompleted = task.status === TaskStatus.Completed
  const isActive = task.id === activeTaskId
  const isRunning = isActive && timerRunning
  const progressPct =
    task.max_sessions > 0
      ? Math.round((task.current_sessions / task.max_sessions) * 100)
      : 0

  return (
    <li
      className={[
        'flex flex-col gap-2 rounded-xl border px-4 py-3.5 transition-colors',
        isActive ? 'border-toro/40 bg-toro/10' : 'border-border bg-card',
        isCompleted ? 'opacity-55' : '',
      ].join(' ')}
    >
      <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
        {/* Selection checkbox */}
        <input
          type="checkbox"
          checked={isSelected}
          onChange={onToggleSelect}
          aria-label={isSelected ? 'Deselect task' : 'Select task'}
          className="border-input text-primary focus:ring-ring h-4 w-4 shrink-0 cursor-pointer rounded focus:ring-2 focus:outline-none"
        />

        {/* Title + meta */}
        <div className="min-w-0 flex-1 basis-40">
          <div className="flex min-w-0 items-center gap-1.5">
            {isActive && (
              <span className="bg-primary text-primary-foreground inline-flex shrink-0 items-center gap-1 rounded-full px-1.5 py-0.5 text-[9px] font-bold tracking-wide uppercase">
                <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-white" />
                Active
              </span>
            )}
            <span
              title={task.name}
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
                className="bg-toro h-full rounded-full transition-all duration-300"
                style={{ width: `${progressPct}%` }}
              />
            </div>
          )}
        </div>

        {/* Session count + actions (wrap to second line on narrow widths) */}
        <div className="ml-auto flex shrink-0 items-center gap-3">
          {/* Session count */}
          <span className="text-muted-foreground text-xs tabular-nums">
            {task.current_sessions}/{task.max_sessions}
          </span>

          <div className="flex items-center gap-0.5">
            {/* Focus */}
            {!isCompleted && (
              <button
                onClick={onSetActive}
                title={timerRunning ? 'Go to running timer' : 'Focus on this task'}
                className={[
                  'shrink-0 rounded p-1 transition-colors',
                  isActive ? 'text-toro' : 'text-muted-foreground hover:text-foreground',
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
                className="text-muted-foreground hover:text-foreground shrink-0 rounded p-1 transition-colors"
              >
                <RotateCcw size={15} />
              </button>
            )}

            {/* Complete / Reopen */}
            {isCompleted ? (
              <button
                onClick={onReset}
                title="Reopen"
                className="text-muted-foreground hover:text-foreground shrink-0 rounded p-1 transition-colors"
              >
                <Undo2 size={15} />
              </button>
            ) : (
              <button
                onClick={onComplete}
                title="Complete"
                className="text-muted-foreground hover:text-toro shrink-0 rounded p-1 transition-colors"
              >
                <Check size={15} />
              </button>
            )}

            {/* Edit */}
            <button
              onClick={onEdit}
              disabled={isRunning}
              title={isRunning ? 'Stop the timer to edit' : 'Edit task'}
              className={[
                'shrink-0 rounded p-1 transition-colors',
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
                'shrink-0 rounded p-1 transition-colors',
                isRunning
                  ? 'text-muted-foreground/40 cursor-not-allowed'
                  : 'text-muted-foreground hover:text-destructive',
              ].join(' ')}
            >
              <Trash2 size={15} />
            </button>
          </div>
        </div>
      </div>
    </li>
  )
}, taskRowPropsEqual)

function taskRowPropsEqual(prev: TaskRowProps, next: TaskRowProps): boolean {
  return (
    prev.task === next.task &&
    prev.timerRunning === next.timerRunning &&
    prev.activeTaskId === next.activeTaskId &&
    prev.isSelected === next.isSelected &&
    prev.onNavigateToTimer === next.onNavigateToTimer &&
    prev.onToggleSelect === next.onToggleSelect
  )
}
