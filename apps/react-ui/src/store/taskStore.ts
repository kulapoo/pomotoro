import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { TaskStatus } from '@/types'
import type { Task, CreateTaskRequest, UpdateTaskRequest } from '@/types'

interface TaskStore {
  tasks: Task[]
  isLoading: boolean
  error: string | null

  loadTasks: () => Promise<void>
  createTask: (req: CreateTaskRequest) => Promise<void>
  updateTask: (req: UpdateTaskRequest) => Promise<void>
  deleteTask: (id: string) => Promise<void>
  completeTask: (id: string) => Promise<void>
  resetTask: (id: string) => Promise<void>
  setActiveTask: (id: string) => Promise<void>
  completeActiveTask: () => Promise<void>
  resetActiveTask: () => Promise<void>
  getActiveTask: () => Task | undefined
}

export const useTaskStore = create<TaskStore>((set, get) => ({
  tasks: [],
  isLoading: false,
  error: null,

  loadTasks: async () => {
    set({ isLoading: true })
    try {
      const tasks = await invoke<Task[]>('get_all_tasks')
      set({ tasks, isLoading: false, error: null })
    } catch (e) {
      set({ error: String(e), isLoading: false })
    }
  },

  createTask: async (req) => {
    try {
      await invoke('create_task', { request: req })
      await get().loadTasks()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  updateTask: async (req) => {
    try {
      await invoke('update_task', { request: req })
      await get().loadTasks()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  deleteTask: async (id) => {
    try {
      await invoke('delete_task', { id })
      await get().loadTasks()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  // Backend: complete_task(task_id: String)
  completeTask: async (id) => {
    try {
      await invoke('complete_task', { task_id: id })
      await get().loadTasks()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  // Backend: reset_task(task_id: String) -> (Timer, Task)
  resetTask: async (id) => {
    try {
      await invoke('reset_task', { task_id: id })
      await get().loadTasks()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  setActiveTask: async (id) => {
    try {
      await invoke('switch_active_task', { task_id: id })
      await get().loadTasks()
    } catch (e) {
      set({ error: String(e) })
      throw e
    }
  },

  completeActiveTask: async () => {
    const activeTask = get().getActiveTask()
    if (!activeTask) return
    try {
      await invoke('complete_task', { task_id: activeTask.id })
      await get().loadTasks()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  resetActiveTask: async () => {
    const activeTask = get().getActiveTask()
    if (!activeTask) return
    try {
      await invoke('reset_task', { task_id: activeTask.id })
      await get().loadTasks()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  getActiveTask: () => get().tasks.find((t) => t.status === TaskStatus.Active),
}))
