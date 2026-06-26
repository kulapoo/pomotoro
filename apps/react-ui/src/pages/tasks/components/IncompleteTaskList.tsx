import { TaskRow } from '@/pages/tasks/components/TaskRow'
import { isTimerRunning, useTimerStore } from '@/pages/timer/useTimer'
import type { Task } from '@/pages/tasks/useTasks'
import type { TaskRowHandlers } from '@/pages/tasks/hooks/useTaskActions'

interface IncompleteTaskListProps {
  tasks: Task[]
  handlers: TaskRowHandlers
  onNavigateToTimer: () => void
  selectedIds: Set<string>
  onToggleSelect: (id: string) => void
}

export function IncompleteTaskList({
  tasks,
  handlers,
  onNavigateToTimer,
  selectedIds,
  onToggleSelect,
}: IncompleteTaskListProps) {
  const timerRunning = useTimerStore((s) => (s.timer ? isTimerRunning(s.timer) : false))

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
          onNavigateToTimer={onNavigateToTimer}
          isSelected={selectedIds.has(task.id)}
          onToggleSelect={() => onToggleSelect(task.id)}
        />
      ))}
    </ul>
  )
}
