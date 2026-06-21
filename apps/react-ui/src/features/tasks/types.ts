import type { Config } from '@/features/settings/types'

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
