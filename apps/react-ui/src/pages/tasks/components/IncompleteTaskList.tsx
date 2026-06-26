import { TaskRow } from '@/pages/tasks/components/TaskRow'
import type { Task } from '@/pages/tasks/useTasks'
import type { TaskRowHandlers } from '@/pages/tasks/hooks/useTaskActions'

interface IncompleteTaskListProps {
  tasks: Task[]
  handlers: TaskRowHandlers
  onNavigateToTimer: () => void
  selectedIds: Set<string>
  onToggleSelect: (id: string) => void
  timerRunning: boolean
  activeTaskId?: string | null
}

export function IncompleteTaskList({
  tasks,
  handlers,
  onNavigateToTimer,
  selectedIds,
  onToggleSelect,
  timerRunning,
  activeTaskId,
}: IncompleteTaskListProps) {

  if (tasks.length === 0) return null

  return (
    <ul className="mb-6 flex flex-col gap-2">
      {tasks.map((task) => (
        <TaskRow
          key={task.id}
          task={task}
          onEdit={() => handlers.onEdit(task)}
          onComplete={() => handlers.onComplete(task)}
          onReset={() => handlers.onReset(task)}
          onDelete={() => handlers.onDelete(task)}
          onSetActive={() => handlers.onSetActive(task)}
          timerRunning={timerRunning}
          activeTaskId={activeTaskId}
          onNavigateToTimer={onNavigateToTimer}
          isSelected={selectedIds.has(task.id)}
          onToggleSelect={() => onToggleSelect(task.id)}
        />
      ))}
    </ul>
  )
}
