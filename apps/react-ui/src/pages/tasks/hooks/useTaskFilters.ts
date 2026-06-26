import { useMemo, useState } from 'react'
import { TaskStatus } from '@/pages/tasks/useTasks'
import type { Task, TaskStatus as TaskStatusType } from '@/pages/tasks/useTasks'

export type StatusFilter = 'all' | TaskStatusType

export function useTaskFilters(tasks: Task[], activeTaskId?: string | null) {
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')

  const filtered = useMemo(() => {
    let result = tasks
    if (statusFilter !== 'all') {
      result = result.filter((t) => t.status === statusFilter)
    }
    if (search.trim()) {
      const q = search.toLowerCase()
      result = result.filter(
        (t) =>
          t.name.toLowerCase().includes(q) ||
          (t.description ?? '').toLowerCase().includes(q) ||
          t.tags.some((tag) => tag.toLowerCase().includes(q)),
      )
    }
    return result
  }, [tasks, statusFilter, search])

  const incomplete = useMemo(
    () => filtered.filter((t) => t.status !== TaskStatus.Completed),
    [filtered],
  )
  const completed = useMemo(
    () => filtered.filter((t) => t.status === TaskStatus.Completed),
    [filtered],
  )

  const visibleIds = useMemo(
    () => [...incomplete, ...completed].map((t) => t.id),
    [incomplete, completed],
  )
  const completedIds = useMemo(() => completed.map((t) => t.id), [completed])

  const total = tasks.length
  const activeCount = useMemo(
    () => (activeTaskId && tasks.some((t) => t.id === activeTaskId) ? 1 : 0),
    [tasks, activeTaskId],
  )
  const completedCount = useMemo(
    () => tasks.filter((t) => t.status === TaskStatus.Completed).length,
    [tasks],
  )

  return {
    search,
    setSearch,
    statusFilter,
    setStatusFilter,
    filtered,
    incomplete,
    completed,
    visibleIds,
    completedIds,
    total,
    activeCount,
    completedCount,
  }
}
