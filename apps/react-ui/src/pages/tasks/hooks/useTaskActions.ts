import { useCallback, useMemo, useState } from 'react'
import { toast } from 'sonner'
import { useTaskStore } from '@/pages/tasks/useTasks'
import type { Task } from '@/pages/tasks/useTasks'
import { isTimerRunning, useTimerStore } from '@/pages/timer/useTimer'
import type { Page } from '@/app/types'

export interface TaskRowHandlers {
  onEdit: (task: Task) => void
  onComplete: (task: Task) => void
  onReset: (task: Task) => void
  onDelete: (task: Task) => void
  onSetActive: (task: Task) => void
}

export function useTaskActions(onNavigate: (page: Page) => void) {
  const activeTask = useTaskStore((s) => s.activeTask)
  const completeTask = useTaskStore((s) => s.completeTask)
  const resetTask = useTaskStore((s) => s.resetTask)
  const resetTasks = useTaskStore((s) => s.resetTasks)
  const deleteTask = useTaskStore((s) => s.deleteTask)
  const setActiveTask = useTaskStore((s) => s.setActiveTask)

  const [showModal, setShowModal] = useState(false)
  const [editTask, setEditTask] = useState<Task | undefined>(undefined)

  const refreshTimer = useCallback(async () => {
    await Promise.all([
      useTaskStore.getState().loadActiveTask(),
      useTimerStore.getState().fetchTimer(),
    ])
  }, [])

  const openCreate = useCallback(() => {
    setEditTask(undefined)
    setShowModal(true)
  }, [])

  const openEdit = useCallback((task: Task) => {
    setEditTask(task)
    setShowModal(true)
  }, [])

  const closeModal = useCallback(() => setShowModal(false), [])

  const handleSetActive = useCallback(
    async (task: Task) => {
      if (activeTask?.id === task.id) {
        onNavigate('timer')
        return
      }
      const ok = await setActiveTask(task.id)
      if (ok) {
        toast.info('Focusing on "' + task.name + '"')
        void refreshTimer()
        const timer = useTimerStore.getState().timer
        if (timer && isTimerRunning(timer)) {
          window.setTimeout(() => useTimerStore.getState().pause(), 200)
        }
        onNavigate('timer')
      }
    },
    [activeTask?.id, onNavigate, refreshTimer, setActiveTask],
  )

  const handleReset = useCallback(
    async (task: Task) => {
      const ok = await resetTask(task.id)
      if (ok) {
        toast.info('Task reopened')
        window.setTimeout(refreshTimer, 50)
      }
    },
    [refreshTimer, resetTask],
  )

  const handleComplete = useCallback(
    async (task: Task) => {
      const ok = await completeTask(task.id)
      if (ok) {
        toast.info('Task completed')
        window.setTimeout(refreshTimer, 50)
      }
    },
    [completeTask, refreshTimer],
  )

  const handleDelete = useCallback(
    async (task: Task) => {
      if (!window.confirm('Delete "' + task.name + '"? This cannot be undone.')) return
      const ok = await deleteTask(task.id)
      if (ok) {
        toast.info('Task deleted')
        window.setTimeout(refreshTimer, 50)
      }
    },
    [deleteTask, refreshTimer],
  )

  const resetMany = useCallback(
    async (ids: string[]) => {
      if (ids.length === 0) return 0
      const ok = await resetTasks(ids)
      if (ok) {
        toast.info(`Reset ${ids.length} task${ids.length === 1 ? '' : 's'}`)
        window.setTimeout(refreshTimer, 50)
        return ids.length
      }
      return 0
    },
    [refreshTimer, resetTasks],
  )

  const handlers = useMemo<TaskRowHandlers>(
    () => ({
      onEdit: openEdit,
      onComplete: handleComplete,
      onReset: handleReset,
      onDelete: handleDelete,
      onSetActive: handleSetActive,
    }),
    [openEdit, handleComplete, handleReset, handleDelete, handleSetActive],
  )

  return {
    showModal,
    editTask,
    openCreate,
    openEdit,
    closeModal,
    handlers,
    resetMany,
  }
}
