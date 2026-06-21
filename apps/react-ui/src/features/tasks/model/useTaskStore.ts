import { create } from 'zustand'
import { invokeCmd } from '@/lib/tauri'
import { BackendError } from '@/lib/errors'
import { logger } from '@/lib/logger'
import { TaskStatus } from '@/features/tasks/types'
import type { Task, CreateTaskRequest, UpdateTaskRequest } from '@/features/tasks/types'

interface TaskStore {
  tasks: Task[]
  isLoading: boolean
  error: BackendError | null
  loadTasks: () => Promise<boolean>
  createTask: (req: CreateTaskRequest) => Promise<boolean>
  updateTask: (req: UpdateTaskRequest) => Promise<boolean>
  deleteTask: (id: string) => Promise<boolean>
  completeTask: (id: string) => Promise<boolean>
  resetTask: (id: string) => Promise<boolean>
  setActiveTask: (id: string) => Promise<boolean>
  completeActiveTask: () => Promise<boolean>
  resetActiveTask: () => Promise<boolean>
  getActiveTask: () => Task | undefined
  clearError: () => void
}

export const useTaskStore = create<TaskStore>((set, get) => ({
  tasks: [],
  isLoading: false,
  error: null,

  loadTasks: async () => {
    set({ isLoading: true })
    try {
      const tasks = await invokeCmd('get_all_tasks')
      set({ tasks, isLoading: false, error: null })
      return true
    } catch (e) {
      logger.error('loadTasks failed', e)
      set({ error: e as BackendError, isLoading: false })
      return false
    }
  },

  createTask: async (req) => {
    try {
      await invokeCmd('create_task', { request: req })
      return true
    } catch (e) {
      logger.error('createTask failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  updateTask: async (req) => {
    try {
      await invokeCmd('update_task', { request: req })
      return true
    } catch (e) {
      logger.error('updateTask failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  deleteTask: async (id) => {
    try {
      await invokeCmd('delete_task', { id })
      return true
    } catch (e) {
      logger.error('deleteTask failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  completeTask: async (id) => {
    try {
      await invokeCmd('complete_task', { task_id: id })
      return true
    } catch (e) {
      logger.error('completeTask failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  resetTask: async (id) => {
    try {
      await invokeCmd('reset_task', { task_id: id })
      return true
    } catch (e) {
      logger.error('resetTask failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  setActiveTask: async (id) => {
    try {
      const oldTask = get().getActiveTask()
      await invokeCmd('switch_active_task', {
        task_id: id,
        old_task_id: oldTask?.id ?? null,
      })
      return true
    } catch (e) {
      logger.error('setActiveTask failed', e)
      set({ error: e as BackendError })
      return false
    }
  },

  completeActiveTask: async () => {
    const active = get().getActiveTask()
    if (!active) return false
    return get().completeTask(active.id)
  },

  resetActiveTask: async () => {
    const active = get().getActiveTask()
    if (!active) return false
    return get().resetTask(active.id)
  },

  getActiveTask: () => get().tasks.find((t) => t.status === TaskStatus.Active),

  clearError: () => set({ error: null }),
}))

/**
 * Reactive selector for the currently active task.
 * Subscribe with `const active = useActiveTask()` so the component re-renders
 * when `tasks` changes — unlike the imperative `getActiveTask()` action.
 */
export const useActiveTask = (): Task | undefined =>
  useTaskStore((s) => s.tasks.find((t) => t.status === TaskStatus.Active))
