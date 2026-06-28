import { useEffect } from 'react'
import { create } from 'zustand'
import { toast } from 'sonner'
import { invokeCmd, onEvent, events } from '@/lib/tauri'
import { BackendError } from '@/lib/errors'
import { logger } from '@/lib/logger'
import { createBatchedLoader } from '@/lib/async'
import type { Config } from '@/pages/settings/useSettings'
import { type TimerStateData, type Timer } from '@/pages/timer/useTimer'
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

export interface TaskActiveChangedPayload {
  old_task_id: string | null
  new_task_id: string
  workflow_result: string
  version: number
  occurred_at: string
  task: Task
}

export interface TaskCompletedPayload {
  task_id: string
  total_sessions: number
  completed_at: string
  version: number
  occurred_at: string
  task: Task
}

export interface TaskResetPayload {
  task_id: string
  name: string | null
  description: string | null
  max_sessions: number | null
  tags: string[] | null
  version: number
  occurred_at: string
  task: Task
}

export interface TasksResetPayload {
  task_ids: string[]
  tasks: Task[]
  version: number
  occurred_at: string
}

export interface TasksCompletedPayload {
  completed_task_ids: string[]
  version: number
  occurred_at: string
}

export interface PhaseCompletedPayload {
  timer: TimerStateData
  task: Task | null
}

export interface TaskAutoAdvancedPayload {
  from_task_id: string
  to_task_id: string
  to_task: Task
  timer: Timer
}

export const MAX_TASKS = 50

interface TaskStore {
  tasks: Task[]
  activeTask: Task | null
  isLoading: boolean
  isBusy: boolean
  error: BackendError | null
  loadTasks: () => Promise<boolean>
  loadActiveTask: () => Promise<boolean>
  applyActiveTask: (task: Task) => void
  applyTaskIfActiveForId: (taskId: string, task: Task, taskPatch?: Partial<Task>) => void
  createTask: (req: CreateTaskRequest) => Promise<boolean>
  updateTask: (req: UpdateTaskRequest) => Promise<boolean>
  deleteTask: (id: string) => Promise<boolean>
  completeTask: (id: string) => Promise<boolean>
  resetTask: (id: string) => Promise<boolean>
  resetTasks: (ids: string[]) => Promise<boolean>
  setActiveTask: (id: string, oldTaskId?: string | null) => Promise<boolean>
  completeActiveTask: (id: string) => Promise<boolean>
  resetActiveTask: (id: string) => Promise<boolean>
  clearError: () => void
}

export const useTaskStore = create<TaskStore>((set, get) => ({
  tasks: [],
  activeTask: null,
  isLoading: false,
  isBusy: false,
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

  loadActiveTask: async () => {
    set({ isLoading: true })
    try {
      const activeTask = await invokeCmd('get_active_task')
      set({ activeTask, isLoading: false, error: null })
      return true
    } catch (e) {
      logger.error('loadActiveTask failed', e)
      set({ error: e as BackendError, isLoading: false })
      return false
    }
  },

  applyActiveTask: (task) => set({ activeTask: task }),

  applyTaskIfActiveForId: (taskId, task, taskPatch = {} as Partial<Task>) => {
    if (get().activeTask?.id === taskId) {
      set({ activeTask: { ...task, ...taskPatch } })
    }
  },

  createTask: async (req) => {
    if (get().tasks.length >= MAX_TASKS) {
      toast.error(`Task limit reached (${MAX_TASKS}). Delete a task first.`)
      return false
    }
    return runBusy(set, get, 'createTask', async () => {
      await invokeCmd('create_task', { request: req })
      return true
    })
  },

  updateTask: async (req) => {
    return runBusy(set, get, 'updateTask', async () => {
      await invokeCmd('update_task', { request: req })
      return true
    })
  },

  deleteTask: async (id) => {
    return runBusy(set, get, 'deleteTask', async () => {
      await invokeCmd('delete_task', { id })
      return true
    })
  },

  completeTask: async (id) => {
    return runBusy(set, get, 'completeTask', async () => {
      await invokeCmd('complete_task', { task_id: id })
      return true
    })
  },

  resetTask: async (id) => {
    return runBusy(set, get, 'resetTask', async () => {
      await invokeCmd('reset_task', { task_id: id })
      return true
    })
  },

  resetTasks: async (ids) => {
    if (ids.length === 0) return false
    return runBusy(set, get, 'resetTasks', async () => {
      await invokeCmd('reset_tasks', { task_ids: ids })
      return true
    })
  },

  setActiveTask: async (id, oldTaskId: string | null = null) => {
    return runBusy(set, get, 'setActiveTask', async () => {
      const oldTask = oldTaskId ? { id: oldTaskId } : await invokeCmd('get_active_task')
      await invokeCmd('switch_active_task', {
        task_id: id,
        old_task_id: oldTask?.id ?? null,
      })
      return true
    })
  },

  completeActiveTask: async (id) => {
    return get().completeTask(id)
  },

  resetActiveTask: async (id) => {
    return get().resetTask(id)
  },

  clearError: () => set({ error: null }),
}))

async function runBusy(
  set: (partial: Partial<TaskStore>) => void,
  get: () => TaskStore,
  name: string,
  fn: () => Promise<boolean>,
): Promise<boolean> {
  if (get().isBusy) return false
  set({ isBusy: true })
  try {
    return await fn()
  } catch (e) {
    logger.error(`${name} failed`, e)
    set({ error: e as BackendError })
    return false
  } finally {
    set({ isBusy: false })
  }
}

export function useTasksEventBus(): void {
  const storeLoadTasks = useTaskStore((s) => s.loadTasks)
  const storeLoadActiveTask = useTaskStore((s) => s.loadActiveTask)
  const applyTaskIfActiveForId = useTaskStore((s) => s.applyTaskIfActiveForId)

  useEffect(() => {
    const reloadTasks = createBatchedLoader(() => storeLoadTasks())
    const reloadActiveTask = createBatchedLoader(() => storeLoadActiveTask())

    const reload = () => {
      window.setTimeout(() => {
        reloadTasks()
        reloadActiveTask()
      }, 500)
    }

    reload()

    const unlisteners: Array<Promise<UnlistenFn>> = [
      onEvent(events.taskListUpdated, reload),
      onEvent(events.taskCompleted, (payload) => {
        applyTaskIfActiveForId(payload.task_id, payload.task)
      }),
      onEvent(events.taskProgressUpdated, (task) => {
        applyTaskIfActiveForId(task.id, task)
      }),
    ]

    return () => {
      for (const p of unlisteners) void p.then((fn) => fn())
    }
  }, [storeLoadTasks, storeLoadActiveTask, applyTaskIfActiveForId])
}
