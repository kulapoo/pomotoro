import { Play, Pause, RotateCcw, SkipForward } from 'lucide-react'

interface TimerControlsProps {
  canResetPhase: boolean
  canPlayPause: boolean
  canSkip: boolean
  running: boolean
  hasContext: boolean
  isBusy: boolean
  isLastBreak: boolean
  onResetPhase: () => void
  onPlayPause: () => void
  onSkip: () => void
}

export function TimerControls({
  canResetPhase,
  canPlayPause,
  canSkip,
  running,
  hasContext,
  isBusy,
  isLastBreak,
  onResetPhase,
  onPlayPause,
  onSkip,
}: TimerControlsProps) {
  return (
    <div className="mt-2 flex items-center gap-5">
      <button
        onClick={onResetPhase}
        disabled={!canResetPhase || isBusy}
        className="text-muted-foreground hover:text-foreground hover:bg-accent rounded-full p-3 transition-colors disabled:cursor-not-allowed disabled:opacity-30"
        title="Restart phase"
      >
        <RotateCcw size={20} />
      </button>

      <button
        onClick={onPlayPause}
        disabled={!canPlayPause}
        className="bg-primary text-primary-foreground flex h-16 w-16 items-center justify-center rounded-full shadow-lg transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
        title={!hasContext ? 'Select a task to start' : undefined}
      >
        {running ? <Pause size={26} /> : <Play size={26} className="ml-1" />}
      </button>

      <button
        onClick={onSkip}
        disabled={!canSkip || isBusy || isLastBreak}
        className="text-muted-foreground hover:text-foreground hover:bg-accent rounded-full p-3 transition-colors disabled:cursor-not-allowed disabled:opacity-30"
        title={isLastBreak ? 'Skip unavailable — task is complete' : 'Skip phase'}
      >
        <SkipForward size={20} />
      </button>
    </div>
  )
}
