import { CheckCircle, RefreshCw } from 'lucide-react'
import { toast } from 'sonner'
import { useTaskStore } from '@/pages/tasks/useTasks'
import { useTimerStore } from '@/pages/timer/useTimer'
import { useTimerSession } from '@/pages/timer/useTimerSession'

export function ActiveTaskActions() {
  const completeActiveTask = useTaskStore((s) => s.completeActiveTask)
  const resetActiveTask = useTaskStore((s) => s.resetActiveTask)
  const loadActiveTask = useTaskStore((s) => s.loadActiveTask)
  const loadTasks = useTaskStore((s) => s.loadTasks)
  const taskBusy = useTaskStore((s) => s.isBusy)
  const timerBusy = useTimerStore((s) => s.isBusy)
  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const { activeTask, isLastBreak, isTaskCompleted } = useTimerSession()
  const isBusy = timerBusy || taskBusy

  if (!activeTask) return null

  const handleCompleteTask = async () => {
    if (isBusy) return

    if (isTaskCompleted) return
    const ok = await completeActiveTask(activeTask.id)
    if (ok) {
      await loadActiveTask()
      await fetchTimer()
      await loadTasks()
      toast.success('Task completed!')
    }
  }

  const handleResetTask = async () => {
    if (isBusy) return
    const ok = await resetActiveTask(activeTask.id)
    if (ok) {
      await Promise.all([fetchTimer(), loadActiveTask()])
      toast.info('Task progress reset')
    }
  }

  return (
    <div className="mt-1 flex items-center gap-3">
      <button
        onClick={handleResetTask}
        disabled={isBusy}
        className={[
          'flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-40',
          isTaskCompleted
            ? 'bg-emerald-500 text-white dark:bg-emerald-600'
            : 'border-border text-muted-foreground hover:text-foreground hover:bg-accent border',
        ].join(' ')}
        title="Reset task progress"
      >
        <RefreshCw size={12} />
        Reset Task
      </button>
      <button
        onClick={handleCompleteTask}
        disabled={isBusy || !!isTaskCompleted}
        className={[
          'flex items-center gap-1.5 rounded-lg px-3.5 py-2 text-xs font-semibold shadow-sm transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40',
          isLastBreak
            ? 'bg-emerald-500 text-white dark:bg-emerald-600'
            : 'bg-primary text-primary-foreground',
        ].join(' ')}
        title={isLastBreak ? 'End this break and finish' : 'Mark task as complete'}
      >
        <CheckCircle size={14} />
        {isLastBreak ? 'Finish Now' : 'Complete Task'}
      </button>
    </div>
  )
}
