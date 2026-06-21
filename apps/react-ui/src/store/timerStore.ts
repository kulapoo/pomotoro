import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { TimerState, Phase } from "@/types";
import type { Timer, TimerStateName } from "@/types";

interface TimerStore {
  timer: Timer | null;
  error: string | null;

  fetchTimer: () => Promise<void>;
  applyTick: (payload: {
    task_id: string;
    phase: string;
    remaining_seconds: number;
  }) => void;
  start: () => Promise<void>;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
  reset: () => Promise<void>;
  skip: () => Promise<void>;
  switchTask: (taskId: string) => Promise<void>;
}

export const useTimerStore = create<TimerStore>((set, get) => ({
  timer: null,
  error: null,

  fetchTimer: async () => {
    try {
      const timer = await invoke<Timer>("get_timer_state");
      set({ timer, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  applyTick: (payload) => {
    const timer = get().timer;
    if (!timer) return;
    const state = timer.state.state;
    if (state === TimerState.Idle || state === TimerState.Paused) return;

    // Reject stale ticks from the wrong task or from a prior phase
    if (payload.task_id !== timer.task_id) return;
    const phaseByState: Partial<Record<TimerStateName, Phase>> = {
      [TimerState.Working]: Phase.Work,
      [TimerState.ShortBreak]: Phase.ShortBreak,
      [TimerState.LongBreak]: Phase.LongBreak,
    };
    if (phaseByState[state] !== payload.phase) return;

    set({
      timer: {
        ...timer,
        state: {
          ...timer.state,
          data: {
            ...timer.state.data,
            remaining_seconds: payload.remaining_seconds,
          },
        },
      },
    });
  },

  start: async () => {
    const taskId = get().timer?.task_id;
    if (!taskId) return;
    try {
      await invoke("start_timer", { task_id: taskId });
      await get().fetchTimer();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  pause: async () => {
    const taskId = get().timer?.task_id;
    if (!taskId) return;
    try {
      await invoke("pause_timer", { task_id: taskId });
      await get().fetchTimer();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  resume: async () => {
    const taskId = get().timer?.task_id;
    if (!taskId) return;
    try {
      await invoke("resume_timer", { task_id: taskId });
      await get().fetchTimer();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  reset: async () => {
    const taskId = get().timer?.task_id;
    if (!taskId) return;
    try {
      // Returns (Timer, Task) tuple - we ignore it and re-fetch
      await invoke("reset_timer", { task_id: taskId });
      await get().fetchTimer();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  skip: async () => {
    const taskId = get().timer?.task_id;
    if (!taskId) return;
    try {
      await invoke("skip_phase", { task_id: taskId });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  switchTask: async (taskId: string) => {
    try {
      await invoke("switch_active_task", { task_id: taskId });
      await get().fetchTimer();
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
