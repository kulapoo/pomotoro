import { RotateCcw } from 'lucide-react'
import { StatBadge } from '@/pages/tasks/components/StatBadge'

interface TasksHeaderProps {
  total: number
  activeCount: number
  completedCount: number
  hasTasks: boolean
  isBusy: boolean
  onResetAll: () => void
}

export function TasksHeader({
  total,
  activeCount,
  completedCount,
  hasTasks,
  isBusy,
  onResetAll,
}: TasksHeaderProps) {
  return (
    <div className="mb-5 flex items-start justify-between">
      <div className="flex items-center gap-3">
        <h1 className="text-2xl font-bold">Tasks</h1>
        {hasTasks && (
          <button
            onClick={onResetAll}
            disabled={isBusy}
            title="Reset every task back to Queued and zero its progress"
            className="border-border text-muted-foreground hover:text-foreground hover:bg-accent flex items-center gap-1.5 rounded-lg border px-2.5 py-1 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-40"
          >
            <RotateCcw size={12} />
            Reset all
          </button>
        )}
      </div>

      <div className="flex gap-2">
        <StatBadge
          label="Total"
          value={total}
          color="border-border bg-card text-foreground"
        />
        <StatBadge
          label="Active"
          value={activeCount}
          color="border-toro/40 bg-toro/10 text-toro"
        />
        <StatBadge
          label="Done"
          value={completedCount}
          color="border-emerald-400/40 bg-emerald-50/50 dark:bg-emerald-950/20 text-emerald-600 dark:text-emerald-400"
        />
      </div>
    </div>
  )
}
