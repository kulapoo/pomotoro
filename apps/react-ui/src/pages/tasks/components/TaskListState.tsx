import { Plus, ListTodo } from 'lucide-react'
import type { BackendError } from '@/lib/errors'

interface TaskListStateProps {
  error: BackendError | null
  isLoading: boolean
  hasTasks: boolean
  hasResults: boolean
  onRetry: () => void
  onOpenCreate: () => void
}

export function TaskListState({
  error,
  isLoading,
  hasTasks,
  hasResults,
  onRetry,
  onOpenCreate,
}: TaskListStateProps) {
  if (error && !isLoading) {
    return (
      <div className="flex flex-col items-center gap-2 py-8 text-center">
        <span className="text-destructive text-sm">{error.message}</span>
        <button
          onClick={onRetry}
          className="border-border hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
        >
          Retry
        </button>
      </div>
    )
  }

  if (isLoading) {
    return <p className="text-muted-foreground py-8 text-center text-sm">Loading…</p>
  }

  if (!error && !hasTasks) {
    return (
      <div className="flex flex-col items-center justify-center px-4 py-16 text-center">
        <div className="bg-muted/60 mb-4 flex h-14 w-14 items-center justify-center rounded-2xl">
          <ListTodo size={26} className="text-muted-foreground" />
        </div>
        <h3 className="mb-1 text-base font-semibold">No tasks yet</h3>
        <p className="text-muted-foreground mb-5 max-w-xs text-sm">
          Create your first task to start focusing. You can edit or delete it any time.
        </p>
        <button
          onClick={onOpenCreate}
          className="bg-primary text-primary-foreground flex items-center gap-2 rounded-xl px-4 py-2.5 text-sm transition-all hover:opacity-90 active:scale-95"
        >
          <Plus size={16} />
          Create task
        </button>
      </div>
    )
  }

  if (!error && hasTasks && !hasResults) {
    return (
      <p className="text-muted-foreground py-12 text-center text-sm">
        No tasks match your search or filter.
      </p>
    )
  }

  return null
}
