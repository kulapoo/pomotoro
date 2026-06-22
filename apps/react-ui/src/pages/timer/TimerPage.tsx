import { useState } from 'react'
import { CheckCircle, RefreshCw, ListTodo } from 'lucide-react'
import { toast } from 'sonner'
import {
  useTimerStore,
  Phase,
  getRemainingSeconds,
  getEffectivePhase,
  isTimerRunning,
  isTimerPaused,
  isTimerIdle,
} from '@/pages/timer/useTimer'
import { TimerRing, CIRC } from '@/pages/timer/components/TimerRing'
import { TimerControls } from '@/pages/timer/components/TimerControls'
import { useTaskStore, useActiveTask, TaskStatus } from '@/pages/tasks/useTasks'
import type { Page } from '@/app/types'
import { formatClock, phaseDuration, DEFAULT_DURATIONS } from '@/lib/duration'

const PHASE_LABEL: Record<Phase, string> = {
  [Phase.Work]: 'Focus',
  [Phase.ShortBreak]: 'Short Break',
  [Phase.LongBreak]: 'Long Break',
}

const PHASE_COLOR: Record<Phase, string> = {
  [Phase.Work]: 'text-indigo-500 dark:text-indigo-400',
  [Phase.ShortBreak]: 'text-emerald-500 dark:text-emerald-400',
  [Phase.LongBreak]: 'text-blue-500 dark:text-blue-400',
}

const PHASE_ARC_COLOR: Record<Phase, string> = {
  [Phase.Work]: '#6366f1',
  [Phase.ShortBreak]: '#10b981',
  [Phase.LongBreak]: '#3b82f6',
}

interface TimerPageProps {
  onNavigate: (page: Page) => void
}

export function TimerPage({ onNavigate }: TimerPageProps) {
  const timer = useTimerStore((s) => s.timer)
  const timerError = useTimerStore((s) => s.error)
  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const start = useTimerStore((s) => s.start)
  const pause = useTimerStore((s) => s.pause)
  const resume = useTimerStore((s) => s.resume)
  const resetPhase = useTimerStore((s) => s.resetPhase)
  const resetTimer = useTimerStore((s) => s.resetTimer)
  const skip = useTimerStore((s) => s.skip)
  const tasks = useTaskStore((s) => s.tasks)
  const completeActiveTask = useTaskStore((s) => s.completeActiveTask)
  const resetActiveTask = useTaskStore((s) => s.resetActiveTask)
  const activeTask = useActiveTask()
  const [isBusy, setIsBusy] = useState(false)

  if (!timer) {
    return (
      <div className="text-muted-foreground flex h-full flex-col items-center justify-center gap-3">
        {timerError ? (
          <>
            <span className="text-destructive text-sm">{timerError.message}</span>
            <button
              onClick={() => void fetchTimer()}
              className="border-border hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
            >
              Retry
            </button>
          </>
        ) : (
          <span>Initializing timer…</span>
        )}
      </div>
    )
  }

  const rawRemaining = getRemainingSeconds(timer)
  const idle = isTimerIdle(timer)

  const contextTask = activeTask ?? tasks.find((t) => t.id === timer.task_id) ?? null
  const timerCfg = contextTask?.config?.timer ?? null

  const idleDuration = timerCfg?.work_duration ?? DEFAULT_DURATIONS.work
  const remaining = idle ? idleDuration : rawRemaining

  const phase = getEffectivePhase(timer)
  const running = isTimerRunning(timer)
  const paused = isTimerPaused(timer)
  const isTaskCompleted = contextTask?.status === TaskStatus.Completed
  const isBreakPhase = phase === Phase.ShortBreak || phase === Phase.LongBreak
  const isLastBreak = !!isTaskCompleted && isBreakPhase
  const canStart = !!timer.task_id && !isTaskCompleted

  const total = phaseDuration(phase, timerCfg)
  const progress = total > 0 ? Math.min(1, Math.max(0, remaining / total)) : 1
  const arcOffset = CIRC * (1 - progress)

  const handlePlayPause = async () => {
    if (running) await pause()
    else if (paused) await resume()
    else await start()
  }

  const handleSkip = async () => {
    if (isBusy || idle || isLastBreak) return
    setIsBusy(true)
    try {
      await skip()
    } finally {
      setIsBusy(false)
    }
  }

  const handleReset = async () => {
    if (isBusy || idle) return
    setIsBusy(true)
    try {
      await resetPhase()
    } finally {
      setIsBusy(false)
    }
  }

  const handleCompleteTask = async () => {
    if (!contextTask || isBusy) return

    if (isLastBreak) {
      setIsBusy(true)
      try {
        const ok = await resetTimer()
        if (ok) toast.success('Task completed!')
      } finally {
        setIsBusy(false)
      }
      return
    }

    if (isTaskCompleted) return
    setIsBusy(true)
    try {
      const ok = await completeActiveTask(contextTask.id)
      if (ok) {
        toast.success('Task completed!')
      }
    } finally {
      setIsBusy(false)
    }
  }

  const handleResetTask = async () => {
    if (!contextTask || isBusy) return
    setIsBusy(true)
    try {
      const ok = await resetActiveTask(contextTask.id)
      if (ok) {
        await fetchTimer()
        toast.info('Task progress reset')
      }
    } finally {
      setIsBusy(false)
    }
  }

  const cycleLen =
    contextTask?.config?.timer?.sessions_until_long_break ??
    DEFAULT_DURATIONS.sessionsUntilLongBreak

  let sessionDots: number[] | null = null
  let dotFilled = 0
  if (contextTask) {
    const hasFixedSessions = (contextTask.max_sessions ?? 0) > 0
    const dotTotal = Math.max(0, hasFixedSessions ? contextTask.max_sessions : cycleLen)
    dotFilled = hasFixedSessions
      ? Math.min(contextTask.current_sessions, contextTask.max_sessions)
      : contextTask.current_sessions % cycleLen
    sessionDots = Array.from({ length: dotTotal }, (_, i) => i)
  }

  return (
    <div className="flex min-h-full flex-col items-center justify-center gap-5 py-10">
      <span
        className={`text-xs font-bold tracking-[0.2em] uppercase ${PHASE_COLOR[phase]}`}
      >
        {PHASE_LABEL[phase]}
      </span>

      <TimerRing
        remainingLabel={formatClock(remaining)}
        arcColor={PHASE_ARC_COLOR[phase]}
        arcOffset={arcOffset}
      />

      {sessionDots && (
        <div className="flex items-center gap-2">
          {sessionDots.map((i) => (
            <div
              key={i}
              className={[
                'h-2.5 w-2.5 rounded-full transition-all duration-300',
                i < dotFilled ? 'bg-indigo-500' : 'bg-muted-foreground/25',
              ].join(' ')}
            />
          ))}
        </div>
      )}

      {contextTask && (
        <div className="bg-card border-border flex max-w-xs items-center gap-2.5 truncate rounded-full border px-4 py-2 shadow-sm">
          {running && (
            <span className="h-2 w-2 shrink-0 animate-pulse rounded-full bg-indigo-500" />
          )}
          <span className="truncate text-sm font-medium">{contextTask.name}</span>
          <span className="text-muted-foreground shrink-0 text-xs tabular-nums">
            {contextTask.current_sessions}/{contextTask.max_sessions}
          </span>
        </div>
      )}

      <TimerControls
        canResetPhase={!idle}
        canPlayPause={!!contextTask && (canStart || running || paused)}
        canSkip={!idle && !isLastBreak}
        running={running}
        hasContext={!!contextTask}
        isBusy={isBusy}
        isLastBreak={isLastBreak}
        onResetPhase={handleReset}
        onPlayPause={handlePlayPause}
        onSkip={handleSkip}
      />

      {!contextTask && !running && !paused && (
        <div className="mt-2 flex flex-col items-center gap-3">
          <div className="bg-muted/60 flex h-12 w-12 items-center justify-center rounded-2xl">
            <ListTodo size={22} className="text-muted-foreground" />
          </div>
          <p className="text-muted-foreground max-w-xs text-center text-sm">
            No task selected. Pick one to start focusing.
          </p>
          <button
            onClick={() => onNavigate('tasks')}
            className="bg-primary text-primary-foreground flex items-center gap-2 rounded-xl px-4 py-2 text-sm transition-all hover:opacity-90 active:scale-95"
          >
            <ListTodo size={15} />
            Choose a task
          </button>
        </div>
      )}

      {isLastBreak ? (
        <span className="text-xs font-medium text-emerald-600 dark:text-emerald-400">
          All sessions complete — this is your final break
        </span>
      ) : (
        <span className="text-muted-foreground text-xs capitalize">
          {isTaskCompleted
            ? 'Task completed'
            : running
              ? 'Running'
              : paused
                ? 'Paused'
                : 'Ready'}
        </span>
      )}

      {contextTask && (
        <div className="mt-1 flex items-center gap-3">
          <button
            onClick={handleResetTask}
            disabled={isBusy}
            className="border-border text-muted-foreground hover:text-foreground hover:bg-accent flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-40"
            title="Reset task progress"
          >
            <RefreshCw size={12} />
            Reset Task
          </button>
          <button
            onClick={handleCompleteTask}
            disabled={isBusy || (!isLastBreak && (isTaskCompleted || !activeTask))}
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
      )}
    </div>
  )
}
