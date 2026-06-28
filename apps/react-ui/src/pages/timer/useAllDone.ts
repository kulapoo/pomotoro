import { useTaskStore, TaskStatus } from '@/pages/tasks/useTasks'

/**
 * True when the active task has just been completed and no other tasks
 * remain incomplete. Used to swap the Timer page for a celebratory
 * "all done" empty state. Relies on completion handlers calling
 * `loadTasks()` so the list is fresh at the moment of evaluation.
 */
export function useAllDone(): boolean {
  const activeTask = useTaskStore((s) => s.activeTask)
  const tasks = useTaskStore((s) => s.tasks)

  if (!activeTask) return false
  if (activeTask.status === TaskStatus.Completed && activeTask.completed_at) return true
  if (tasks.length === 0) return false

  return tasks.every((t) => t.status === TaskStatus.Completed || !!t.completed_at)
}
