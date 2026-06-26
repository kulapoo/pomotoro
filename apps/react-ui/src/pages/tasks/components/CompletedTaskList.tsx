import { ChevronRight } from 'lucide-react'
import { TaskRow } from '@/pages/tasks/components/TaskRow'
import { useTaskStore } from '@/pages/tasks/useTasks'
import type { Task } from '@/pages/tasks/useTasks'
import { isTimerRunning, useTimerStore } from '@/pages/timer/useTimer'
import type { TaskRowHandlers } from '@/pages/tasks/hooks/useTaskActions'

interface CompletedTaskListProps {
  tasks: Task[]
  handlers: TaskRowHandlers
  onNavigateToTimer: () => void
  selectedIds: Set<string>
  isAllCompletedSelected: boolean
  onToggleSelect: (id: string) => void
  onToggleSelectAllCompleted: () => void
}

export function CompletedTaskList({
  tasks,
  handlers,
  onNavigateToTimer,
  selectedIds,
  isAllCompletedSelected,
  onToggleSelect,
  onToggleSelectAllCompleted,
}: CompletedTaskListProps) {
  const timerRunning = useTimerStore((s) => (s.timer ? isTimerRunning(s.timer) : false))
  const setActiveTask = useTaskStore((s) => s.setActiveTask)

  if (tasks.length === 0) return null

  return (
    <details className="group">
      <summary className="text-muted-foreground mb-3 flex cursor-pointer list-none items-center gap-1 text-xs font-semibold tracking-wider uppercase select-none">
        <ChevronRight size={14} className="transition-transform group-open:rotate-90" />
        Completed ({tasks.length})
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation()
            onToggleSelectAllCompleted()
          }}
          title={
            isAllCompletedSelected
              ? 'Clear completed selection'
              : 'Select all completed tasks'
          }
          className="text-muted-foreground hover:text-foreground ml-auto rounded text-xs font-medium tracking-normal normal-case underline transition-colors"
        >
          {isAllCompletedSelected ? 'Clear' : 'Select all'}
        </button>
      </summary>
      <ul className="flex flex-col gap-2">
        {tasks.map((task) => (
          <TaskRow
            key={task.id}
            task={task}
            onEdit={() => handlers.onEdit(task)}
            onComplete={() => handlers.onComplete(task)}
            onReset={() => handlers.onReset(task)}
            onDelete={() => handlers.onDelete(task)}
            onSetActive={() => void setActiveTask(task.id)}
            timerRunning={timerRunning}
            onNavigateToTimer={onNavigateToTimer}
            isSelected={selectedIds.has(task.id)}
            onToggleSelect={() => onToggleSelect(task.id)}
          />
        ))}
      </ul>
    </details>
  )
}
