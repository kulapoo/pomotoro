import { Search } from 'lucide-react'
import type { StatusFilter } from '@/pages/tasks/hooks/useTaskFilters'

interface TaskSearchBarProps {
  search: string
  onSearchChange: (value: string) => void
  statusFilter: StatusFilter
  onStatusChange: (value: StatusFilter) => void
}

export function TaskSearchBar({
  search,
  onSearchChange,
  statusFilter,
  onStatusChange,
}: TaskSearchBarProps) {
  return (
    <div className="mb-6 flex flex-wrap gap-2">
      <div className="relative flex-1">
        <Search
          size={14}
          className="text-muted-foreground absolute top-1/2 left-3 -translate-y-1/2"
        />
        <input
          type="text"
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder="Search tasks…"
          className="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring w-full rounded-xl border py-2 pr-4 pl-9 text-sm focus:ring-2 focus:outline-none"
        />
      </div>
      <select
        value={statusFilter}
        onChange={(e) => onStatusChange(e.target.value as StatusFilter)}
        className="border-input bg-background text-foreground focus:ring-ring rounded-xl border px-3 py-2 text-sm focus:ring-2 focus:outline-none"
      >
        <option value="all">All</option>
        <option value="Active">Active</option>
        <option value="Queued">Queued</option>
        <option value="Paused">Paused</option>
        <option value="Completed">Completed</option>
      </select>
    </div>
  )
}
