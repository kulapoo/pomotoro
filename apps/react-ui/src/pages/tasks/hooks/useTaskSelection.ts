import { useCallback, useEffect, useMemo, useState } from 'react'
import type { StatusFilter } from '@/pages/tasks/hooks/useTaskFilters'

interface UseTaskSelectionArgs {
  visibleIds: string[]
  completedIds: string[]
  search: string
  statusFilter: StatusFilter
}

export function useTaskSelection({
  visibleIds,
  completedIds,
  search,
  statusFilter,
}: UseTaskSelectionArgs) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())

  const clearSelection = useCallback(() => setSelectedIds(new Set()), [])

  useEffect(() => {
    clearSelection()
  }, [search, statusFilter, clearSelection])

  const toggleSelect = useCallback((id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }, [])

  const toggleSelectAllVisible = useCallback(() => {
    setSelectedIds((prev) => {
      const allSelected = visibleIds.length > 0 && visibleIds.every((id) => prev.has(id))
      if (allSelected) return new Set()
      return new Set(visibleIds)
    })
  }, [visibleIds])

  const toggleSelectAllCompleted = useCallback(() => {
    setSelectedIds((prev) => {
      const next = new Set(prev)
      const allSelected =
        completedIds.length > 0 && completedIds.every((id) => next.has(id))
      for (const id of completedIds) {
        if (allSelected) next.delete(id)
        else next.add(id)
      }
      return next
    })
  }, [completedIds])

  const isAllVisibleSelected =
    visibleIds.length > 0 && visibleIds.every((id) => selectedIds.has(id))

  const isAllCompletedSelected =
    completedIds.length > 0 && completedIds.every((id) => selectedIds.has(id))

  const selectedArray = useMemo(() => [...selectedIds], [selectedIds])

  return {
    selectedIds,
    selectedArray,
    size: selectedIds.size,
    isAllVisibleSelected,
    isAllCompletedSelected,
    toggleSelect,
    toggleSelectAllVisible,
    toggleSelectAllCompleted,
    clearSelection,
  }
}
