import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Toaster, toast } from "sonner";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { ScreenBlocker } from "@/components/ScreenBlocker";
import { Sidebar } from "@/components/Sidebar";
import { TimerPage } from "@/pages/TimerPage";
import { TasksPage } from "@/pages/TasksPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { useTimerStore } from "@/store/timerStore";
import { useTaskStore } from "@/store/taskStore";
import { useSettingsStore } from "@/store/settingsStore";
import { AppEvents, TaskStatus } from "@/types";
import type { Page } from "@/types";

export function App() {
  const [page, setPage] = useState<Page>("timer");
  const [ready, setReady] = useState(false);
  const [isBlocking, setIsBlocking] = useState(false);
  const [blockingMessage, setBlockingMessage] = useState("");

  const fetchTimer = useTimerStore((s) => s.fetchTimer);
  const applyTick = useTimerStore((s) => s.applyTick);
  const loadTasks = useTaskStore((s) => s.loadTasks);
  const loadConfig = useSettingsStore((s) => s.loadConfig);
  const notifConfig = useSettingsStore((s) => s.config?.notification);

  const toasterPosition = useMemo(() => {
    switch (notifConfig?.notification_position) {
      case "TopLeft":
        return "top-left" as const;
      case "BottomRight":
        return "bottom-right" as const;
      case "BottomLeft":
        return "bottom-left" as const;
      case "Center":
        return "top-center" as const;
      default:
        return "top-right" as const;
    }
  }, [notifConfig?.notification_position]);

  const handleDismissBlocker = useCallback(() => {
    setIsBlocking(false);
    invoke("deactivate_screen_block").catch(() => {});
  }, []);

  useEffect(() => {
    let cancelled = false;

    const init = async () => {
      await Promise.allSettled([fetchTimer(), loadTasks(), loadConfig()]);
      if (!cancelled) setReady(true);
    };

    init();

    const unlistenPromise = listen(AppEvents.AppInitialized, () => init());

    // Real-time task updates via Tauri events
    const unlistenTaskList = listen(AppEvents.TaskListUpdated, () =>
      loadTasks(),
    );
    const unlistenTaskActive = listen(AppEvents.TaskActiveChanged, () => {
      loadTasks();
      fetchTimer();
    });
    const unlistenTaskCompleted = listen(AppEvents.TaskCompleted, () =>
      loadTasks(),
    );
    const unlistenTaskProgress = listen(AppEvents.TaskProgressUpdated, () =>
      loadTasks(),
    );
    const unlistenTaskAutoAdvanced = listen(
      AppEvents.TaskAutoAdvanced,
      () => {
        fetchTimer();
        loadTasks();
        toast.success("Switched to next incomplete task");
      },
    );

    // Live countdown updates — tick events carry the authoritative remaining_seconds
    const unlistenTimerTick = listen(AppEvents.TimerTick, (event) => {
      applyTick(
        event.payload as {
          task_id: string;
          phase: string;
          remaining_seconds: number;
        },
      );
    });

    // State-change events re-fetch from backend for accurate full state
    const unlistenTimerStatus = listen(AppEvents.TimerStatusChanged, () =>
      fetchTimer(),
    );
    const unlistenPhaseCompleted = listen(AppEvents.TimerPhaseCompleted, () => {
      fetchTimer();
      loadTasks();
    });
    const unlistenPhaseSkipped = listen(AppEvents.TimerPhaseSkipped, () => {
      fetchTimer();
      loadTasks();
    });
    const unlistenTimerReset = listen(AppEvents.TimerReset, () => fetchTimer());
    const unlistenTimerPaused = listen(AppEvents.TimerPaused, () =>
      fetchTimer(),
    );
    const unlistenTimerResumed = listen(AppEvents.TimerResumed, () =>
      fetchTimer(),
    );

    const unlistenScreenBlocker = listen(
      AppEvents.ScreenBlockerActivate,
      (event) => {
        const payload = event.payload as { message: string };
        setBlockingMessage(payload.message);
        setIsBlocking(true);
        invoke("activate_screen_block").catch(() => {});
      },
    );

    return () => {
      cancelled = true;
      unlistenPromise.then((fn) => fn());
      unlistenTaskList.then((fn) => fn());
      unlistenTaskActive.then((fn) => fn());
      unlistenTaskCompleted.then((fn) => fn());
      unlistenTaskProgress.then((fn) => fn());
      unlistenTaskAutoAdvanced.then((fn) => fn());
      unlistenTimerTick.then((fn) => fn());
      unlistenTimerStatus.then((fn) => fn());
      unlistenPhaseCompleted.then((fn) => fn());
      unlistenPhaseSkipped.then((fn) => fn());
      unlistenTimerReset.then((fn) => fn());
      unlistenTimerPaused.then((fn) => fn());
      unlistenTimerResumed.then((fn) => fn());
      unlistenScreenBlocker.then((fn) => fn());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Tab") {
        e.preventDefault();
        const { tasks, setActiveTask } = useTaskStore.getState();
        const incomplete = tasks.filter(
          (t) => t.status !== TaskStatus.Completed,
        );
        if (incomplete.length <= 1) return;
        const activeIdx = incomplete.findIndex(
          (t) => t.status === TaskStatus.Active,
        );
        const nextIdx = e.shiftKey
          ? (activeIdx - 1 + incomplete.length) % incomplete.length
          : (activeIdx + 1) % incomplete.length;
        const next = incomplete[nextIdx];
        if (next) setActiveTask(next.id);
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, []);

  if (!ready) {
    return (
      <div className="flex h-screen w-full items-center justify-center bg-gradient-to-br from-indigo-50 via-white to-purple-50 dark:from-gray-950 dark:via-gray-900 dark:to-indigo-950">
        <span className="text-sm text-muted-foreground animate-pulse">
          Starting Pomotoro…
        </span>
      </div>
    );
  }

  return (
    <>
      <div className="flex h-screen w-full overflow-hidden font-sans bg-gradient-to-br from-indigo-50 via-white to-purple-50 dark:from-gray-950 dark:via-gray-900 dark:to-indigo-950 text-foreground transition-colors duration-300">
        <Toaster position={toasterPosition} richColors />
        <Sidebar currentPage={page} onNavigate={setPage} />
        <main className="flex-1 overflow-y-auto p-6 md:p-10">
          {page === "timer" && (
            <ErrorBoundary>
              <TimerPage />
            </ErrorBoundary>
          )}
          {page === "tasks" && (
            <ErrorBoundary>
              <TasksPage onNavigate={setPage} />
            </ErrorBoundary>
          )}
          {page === "settings" && (
            <ErrorBoundary>
              <SettingsPage />
            </ErrorBoundary>
          )}
        </main>
      </div>
      {isBlocking && (
        <ScreenBlocker
          message={blockingMessage}
          onDismiss={handleDismissBlocker}
        />
      )}
    </>
  );
}
