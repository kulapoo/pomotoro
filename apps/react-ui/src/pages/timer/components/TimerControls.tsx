import { useState } from 'react'
import { Play, Pause, RotateCcw, SkipForward } from 'lucide-react'
import { useTimerStore } from '@/pages/timer/useTimer'
import { useTimerSession } from '@/pages/timer/useTimerSession'

export function TimerControls() {
  const start = useTimerStore((s) => s.start)
  const pause = useTimerStore((s) => s.pause)
  const resume = useTimerStore((s) => s.resume)
  const resetPhase = useTimerStore((s) => s.resetPhase)
  const skip = useTimerStore((s) => s.skip)
  const { activeTask, idle, running, paused, isLastBreak, canPlayPause } =
    useTimerSession()
  const [isBusy, setIsBusy] = useState(false)

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

  return (
    <div className="mt-2 flex items-center gap-5">
      <button
        onClick={handleReset}
        disabled={idle || isBusy}
        className="text-muted-foreground hover:text-foreground hover:bg-accent rounded-full p-3 transition-colors disabled:cursor-not-allowed disabled:opacity-30"
        title="Restart phase"
      >
        <RotateCcw size={20} />
      </button>

      <button
        onClick={handlePlayPause}
        disabled={!canPlayPause}
        className="bg-primary text-primary-foreground flex h-16 w-16 items-center justify-center rounded-full shadow-lg transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
        title={!activeTask ? 'Select a task to start' : undefined}
      >
        {running ? <Pause size={26} /> : <Play size={26} className="ml-1" />}
      </button>

      <button
        onClick={handleSkip}
        disabled={idle || isBusy || isLastBreak}
        className="text-muted-foreground hover:text-foreground hover:bg-accent rounded-full p-3 transition-colors disabled:cursor-not-allowed disabled:opacity-30"
        title={isLastBreak ? 'Skip unavailable — task is complete' : 'Skip phase'}
      >
        <SkipForward size={20} />
      </button>
    </div>
  )
}
