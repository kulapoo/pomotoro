import { CheckCircle, RefreshCw } from 'lucide-react'

interface ActiveTaskActionsProps {
  onResetTask: () => void
  onCompleteTask: () => void
  isBusy: boolean
  isLastBreak: boolean
  isTaskCompleted: boolean
  hasActiveTask: boolean
}

export function ActiveTaskActions({
  onResetTask,
  onCompleteTask,
  isBusy,
  isLastBreak,
  isTaskCompleted,
  hasActiveTask,
}: ActiveTaskActionsProps) {
  return (
    <div className="mt-1 flex items-center gap-3">
      <button
        onClick={onResetTask}
        disabled={isBusy}
        className="border-border text-muted-foreground hover:text-foreground hover:bg-accent flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-40"
        title="Reset task progress"
      >
        <RefreshCw size={12} />
        Reset Task
      </button>
      <button
        onClick={onCompleteTask}
        disabled={isBusy || (!isLastBreak && (isTaskCompleted || !hasActiveTask))}
        className={[
          'flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-40',
          isLastBreak
            ? 'hover:bg-accent border-emerald-500 text-emerald-600 dark:text-emerald-400'
            : 'border-border text-muted-foreground hover:text-foreground hover:bg-accent',
        ].join(' ')}
        title={isLastBreak ? 'End this break and finish' : 'Mark task as complete'}
      >
        <CheckCircle size={12} />
        {isLastBreak ? 'Finish Now' : 'Complete Task'}
      </button>
    </div>
  )
}
