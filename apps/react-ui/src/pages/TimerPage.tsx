import { useState, useEffect } from "react";
import {
  Play,
  Pause,
  RotateCcw,
  SkipForward,
  CheckCircle,
  RefreshCw,
} from "lucide-react";
import { toast } from "sonner";
import { useTimerStore } from "@/store/timerStore";
import { useTaskStore } from "@/store/taskStore";
import {
  Phase,
  TaskStatus,
  getRemainingSeconds,
  getEffectivePhase,
  isTimerRunning,
  isTimerPaused,
  isTimerIdle,
} from "@/types";
import type { TimerConfiguration } from "@/types";

const RING_R = 90;
const CIRC = 2 * Math.PI * RING_R;

function fmtTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function getPhaseDuration(
  phase: Phase,
  cfg: TimerConfiguration | null | undefined,
): number {
  switch (phase) {
    case Phase.ShortBreak:
      return cfg?.short_break_duration ?? 300;
    case Phase.LongBreak:
      return cfg?.long_break_duration ?? 900;
    default:
      return cfg?.work_duration ?? 1500;
  }
}

const PHASE_LABEL: Record<Phase, string> = {
  [Phase.Work]: "Focus",
  [Phase.ShortBreak]: "Short Break",
  [Phase.LongBreak]: "Long Break",
};

const PHASE_COLOR: Record<Phase, string> = {
  [Phase.Work]: "text-indigo-500 dark:text-indigo-400",
  [Phase.ShortBreak]: "text-emerald-500 dark:text-emerald-400",
  [Phase.LongBreak]: "text-blue-500 dark:text-blue-400",
};

const PHASE_ARC_COLOR: Record<Phase, string> = {
  [Phase.Work]: "#6366f1", // indigo-500
  [Phase.ShortBreak]: "#10b981", // emerald-500
  [Phase.LongBreak]: "#3b82f6", // blue-500
};

export function TimerPage() {
  const {
    timer,
    error: timerError,
    start,
    pause,
    resume,
    reset,
    skip,
    fetchTimer,
  } = useTimerStore();
  const { tasks, getActiveTask, completeActiveTask, resetActiveTask } =
    useTaskStore();
  const activeTask = getActiveTask();
  const [isBusy, setIsBusy] = useState(false);

  // Surface operation errors (not init errors) as toasts and clear them
  useEffect(() => {
    if (!timerError) return;
    if (!useTimerStore.getState().timer) return;
    toast.error(timerError);
    useTimerStore.setState({ error: null });
  }, [timerError]);

  if (!timer) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 text-muted-foreground">
        {timerError ? (
          <>
            <span className="text-sm text-destructive">{timerError}</span>
            <button
              onClick={fetchTimer}
              className="text-xs px-3 py-1.5 rounded-lg border border-border hover:bg-accent transition-colors"
            >
              Retry
            </button>
          </>
        ) : (
          <span>Initializing timer…</span>
        )}
      </div>
    );
  }

  const rawRemaining = getRemainingSeconds(timer);
  const idle = isTimerIdle(timer);

  // Resolve the task whose config drives durations (active task or timer's task_id fallback)
  const contextTask =
    activeTask ?? tasks.find((t) => t.id === timer.task_id) ?? null;
  const timerCfg = contextTask?.config?.timer ?? null;

  const idleDuration = timerCfg?.work_duration ?? 1500;
  const remaining = idle ? idleDuration : rawRemaining;

  const phase = getEffectivePhase(timer);
  const running = isTimerRunning(timer);
  const paused = isTimerPaused(timer);
  const isTaskCompleted = contextTask?.status === TaskStatus.Completed;
  const canStart = !!timer.task_id && !isTaskCompleted;

  // Progress ring — fraction of current phase remaining
  const phaseDuration = getPhaseDuration(phase, timerCfg);
  const progress =
    phaseDuration > 0 ? Math.min(1, Math.max(0, remaining / phaseDuration)) : 1;
  const arcOffset = CIRC * (1 - progress);

  const handlePlayPause = async () => {
    try {
      if (running) await pause();
      else if (paused) await resume();
      else await start();
    } catch (e) {
      console.error(e);
      toast.error("Failed to control timer");
    }
  };

  const handleSkip = async () => {
    if (isBusy || idle) return;
    setIsBusy(true);
    try {
      await skip();
    } catch (e) {
      console.error(e);
      toast.error("Failed to skip phase");
    } finally {
      setIsBusy(false);
    }
  };

  const handleReset = async () => {
    if (isBusy || idle) return;
    setIsBusy(true);
    try {
      await reset();
    } catch (e) {
      console.error(e);
      toast.error("Failed to reset timer");
    } finally {
      setIsBusy(false);
    }
  };

  const handleCompleteTask = async () => {
    if (!contextTask || isTaskCompleted || isBusy) return;
    setIsBusy(true);
    try {
      await completeActiveTask();
      await fetchTimer();
      toast.success("Task completed!");
    } catch (e) {
      console.error(e);
      toast.error("Failed to complete task");
    } finally {
      setIsBusy(false);
    }
  };

  const handleResetTask = async () => {
    if (!activeTask || isBusy) return;
    setIsBusy(true);
    try {
      await resetActiveTask();
      await fetchTimer();
      toast.info("Task progress reset");
    } catch (e) {
      console.error(e);
      toast.error("Failed to reset task");
    } finally {
      setIsBusy(false);
    }
  };

  const cycleLen = contextTask?.config?.timer?.sessions_until_long_break ?? 4;

  let sessionDots: number[] | null = null;
  let dotFilled = 0;
  if (contextTask) {
    const hasFixedSessions =
      !contextTask.default && (contextTask.max_sessions ?? 0) > 0;
    const dotTotal = Math.max(
      0,
      hasFixedSessions ? contextTask.max_sessions : cycleLen,
    );
    dotFilled = hasFixedSessions
      ? Math.min(contextTask.current_sessions, contextTask.max_sessions)
      : contextTask.current_sessions % cycleLen;
    sessionDots = Array.from({ length: dotTotal }, (_, i) => i);
  }

  return (
    <div className="flex flex-col items-center justify-center min-h-full gap-5 py-10">
      {/* Phase label */}
      <span
        className={`text-xs font-bold uppercase tracking-[0.2em] ${PHASE_COLOR[phase]}`}
      >
        {PHASE_LABEL[phase]}
      </span>

      {/* Timer ring + digits */}
      <div
        className="relative flex items-center justify-center"
        style={{ width: 248, height: 248 }}
      >
        <svg
          className="-rotate-90"
          viewBox="0 0 200 200"
          width={248}
          height={248}
        >
          {/* Background track */}
          <circle
            cx="100"
            cy="100"
            r={RING_R}
            fill="none"
            stroke="currentColor"
            strokeWidth="5"
            className="text-muted-foreground/15"
          />
          {/* Countdown arc */}
          <circle
            cx="100"
            cy="100"
            r={RING_R}
            fill="none"
            strokeWidth="5"
            strokeLinecap="round"
            strokeDasharray={CIRC}
            style={{
              stroke: PHASE_ARC_COLOR[phase],
              strokeDashoffset: arcOffset,
              transition: "stroke-dashoffset 1s linear, stroke 0.4s ease",
            }}
          />
        </svg>
        <span className="absolute text-7xl font-mono font-bold tabular-nums tracking-tight select-none">
          {fmtTime(remaining)}
        </span>
      </div>

      {/* Session progress dots */}
      {sessionDots && (
        <div className="flex items-center gap-2">
          {sessionDots.map((i) => (
            <div
              key={i}
              className={[
                "w-2.5 h-2.5 rounded-full transition-all duration-300",
                i < dotFilled ? "bg-indigo-500" : "bg-muted-foreground/25",
              ].join(" ")}
            />
          ))}
        </div>
      )}

      {/* Active task pill */}
      {contextTask && !contextTask.default && (
        <div className="flex items-center gap-2.5 px-4 py-2 rounded-full bg-card border border-border shadow-sm max-w-xs truncate">
          {running && (
            <span className="w-2 h-2 rounded-full bg-indigo-500 animate-pulse shrink-0" />
          )}
          <span className="text-sm font-medium truncate">
            {contextTask.name}
          </span>
          <span className="text-xs text-muted-foreground tabular-nums shrink-0">
            {contextTask.current_sessions}/{contextTask.max_sessions}
          </span>
        </div>
      )}

      {/* Primary controls */}
      <div className="flex items-center gap-5 mt-2">
        <button
          onClick={handleReset}
          disabled={idle || isBusy}
          className="p-3 rounded-full text-muted-foreground hover:text-foreground hover:bg-accent transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="Reset timer"
        >
          <RotateCcw size={20} />
        </button>

        <button
          onClick={handlePlayPause}
          disabled={!canStart && !running && !paused}
          className="flex items-center justify-center w-16 h-16 rounded-full bg-primary text-primary-foreground shadow-lg hover:opacity-90 active:scale-95 transition-all disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {running ? <Pause size={26} /> : <Play size={26} className="ml-1" />}
        </button>

        <button
          onClick={handleSkip}
          disabled={idle || isBusy}
          className="p-3 rounded-full text-muted-foreground hover:text-foreground hover:bg-accent transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="Skip phase"
        >
          <SkipForward size={20} />
        </button>
      </div>

      {/* No-task hint */}
      {!contextTask && !running && !paused && (
        <span className="text-xs text-muted-foreground">
          Select a task to start
        </span>
      )}

      {/* State label */}
      <span className="text-xs text-muted-foreground capitalize">
        {isTaskCompleted
          ? "Task completed"
          : running
            ? "Running"
            : paused
              ? "Paused"
              : "Ready"}
      </span>

      {/* Task action buttons */}
      {contextTask && (
        <div className="flex items-center gap-3 mt-1">
          <button
            onClick={handleResetTask}
            disabled={isBusy || !activeTask}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            title="Reset task progress"
          >
            <RefreshCw size={12} />
            Reset Task
          </button>
          {!contextTask.default && (
            <button
              onClick={handleCompleteTask}
              disabled={isTaskCompleted || isBusy || !activeTask}
              className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              title="Mark task as complete"
            >
              <CheckCircle size={12} />
              Complete Task
            </button>
          )}
        </div>
      )}
    </div>
  );
}
