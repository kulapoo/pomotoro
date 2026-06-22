import { useEffect } from 'react'
import { create } from 'zustand'
import { invokeCmd, onEvent, events } from '@/lib/tauri'
import { BackendError } from '@/lib/errors'
import { logger } from '@/lib/logger'
import { createBatchedLoader } from '@/lib/async'
import type { Config } from '@/pages/settings/useSettings'
import type { UnlistenFn } from '@tauri-apps/api/event'

export const TaskStatus = {
  Active: 'Active',
  Queued: 'Queued',
  Completed: 'Completed',
  Paused: 'Paused',
} as const
export type TaskStatus = (typeof TaskStatus)[keyof typeof TaskStatus]

export interface Task {
  id: string
  name: string
  description: string | null
  max_sessions: number
  current_sessions: number
  tags: string[]
  config: Config
  created_at: string
  updated_at: string
  completed_at: string | null
  status: TaskStatus
}

export interface CreateTaskRequest {
  name: string
  description?: string
  max_sessions: number
  tags: string[]
  work_duration?: number
  short_break_duration?: number
  long_break_duration?: number
  sessions_until_long_break?: number
}

export interface UpdateTaskRequest {
  id: string
  name?: string
  description?: string
  max_sessions?: number
  tags?: string[]
  work_duration?: number
  short_break_duration?: number
  long_break_duration?: number
  sessions_until_long_break?: number
}

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
  completeActiveTask: (id: string) => Promise<boolean>
  resetActiveTask: (id: string) => Promise<boolean>
  getActiveTask: () => Promise<Task | null>
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
      const oldTask = await get().getActiveTask()
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

  completeActiveTask: async (id) => {
    return get().completeTask(id)
  },

  resetActiveTask: async (id) => {
    return get().resetTask(id)
  },

  getActiveTask: async () => {
    try {
      return await invokeCmd('get_active_task')
    } catch (e) {
      logger.error('getActiveTask failed', e)
      set({ error: e as BackendError })
      return null
    }
  },

  clearError: () => set({ error: null }),
}))

export const useActiveTask = (): Task | undefined =>
  useTaskStore((s) => s.tasks.find((t) => t.status === TaskStatus.Active))

export function useTasksEventBus(): void {
  const loadTasks = useTaskStore((s) => s.loadTasks)

  useEffect(() => {
    const reloadTasks = createBatchedLoader(() => loadTasks())

    reloadTasks()

    const unlisteners: Array<Promise<UnlistenFn>> = [
      onEvent(events.taskListUpdated, reloadTasks),
      onEvent(events.taskCompleted, reloadTasks),
      onEvent(events.taskProgressUpdated, reloadTasks),
      onEvent(events.taskAutoAdvanced, reloadTasks),
      onEvent(events.appInitialized, reloadTasks),
    ]

    return () => {
      for (const p of unlisteners) void p.then((fn) => fn())
    }
  }, [loadTasks])
}
