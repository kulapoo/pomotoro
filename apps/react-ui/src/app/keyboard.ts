import { useEffect } from 'react'
import { useTaskStore, TaskStatus } from '@/pages/tasks/useTasks'

/**
 * Global keyboard shortcut: Ctrl/Meta+Tab cycles incomplete tasks.
 * Shift+Ctrl/Meta+Tab cycles backwards.
 */
export function useTaskCycling(): void {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey) || e.key !== 'Tab') return
      e.preventDefault()
      const { tasks, setActiveTask } = useTaskStore.getState()
      const incomplete = tasks.filter((t) => t.status !== TaskStatus.Completed)
      if (incomplete.length <= 1) return
      const activeIdx = incomplete.findIndex((t) => t.status === TaskStatus.Active)
      const nextIdx = e.shiftKey
        ? (activeIdx - 1 + incomplete.length) % incomplete.length
        : (activeIdx + 1) % incomplete.length
      const next = incomplete[nextIdx]
      if (next) void setActiveTask(next.id)
    }
    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [])
}
