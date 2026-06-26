import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { BackendError } from '@/lib/errors'
import type {
  Timer,
  TickPayload,
  TimerStateData,
  TimerStatusChangedPayload,
  PhaseSkippedPayload,
} from '@/pages/timer/useTimer'
import type { Config, PlaybackHandle } from '@/pages/settings/useSettings'
import type {
  Task,
  CreateTaskRequest,
  UpdateTaskRequest,
  TaskActiveChangedPayload,
  TaskCompletedPayload,
  TaskAutoAdvancedPayload,
} from '@/pages/tasks/useTasks'

/**
 * Single source of truth for Tauri command names.
 * These strings MUST match the `#[tauri::command]` fn names registered in
 * apps/tauri-app/src/lib.rs (generate_handler!). Drift = compile error here.
 */
export const commands = {
  getTimerState: 'get_timer_state',
  getTask: 'get_task',
  getActiveTask: 'get_active_task',

  startTimer: 'start_timer',
  pauseTimer: 'pause_timer',
  resumeTimer: 'resume_timer',
  resetTimer: 'reset_timer',
  resetTimerPhase: 'reset_timer_phase',
  skipPhase: 'skip_phase',
  switchActiveTask: 'switch_active_task',
  getAllTasks: 'get_all_tasks',
  createTask: 'create_task',
  updateTask: 'update_task',
  deleteTask: 'delete_task',
  completeTask: 'complete_task',
  resetTask: 'reset_task',
  getGlobalConfig: 'get_global_config',
  saveGlobalConfig: 'save_global_config',
  resetConfigToDefaults: 'reset_config_to_defaults',
  testAudioPreview: 'test_audio_preview',
  openDataDirectory: 'open_data_directory',
  clearAllData: 'clear_all_data',
  activateScreenBlock: 'activate_screen_block',
  deactivateScreenBlock: 'deactivate_screen_block',
} as const

export type CommandName = (typeof commands)[keyof typeof commands]

interface CommandMap {
  get_timer_state: { args: void; ret: Timer }
  start_timer: { args: { task_id: string }; ret: Timer }
  pause_timer: { args: { task_id: string; remaining_seconds: number }; ret: Timer }
  resume_timer: { args: { task_id: string }; ret: Timer }
  reset_timer: { args: { task_id: string }; ret: Timer }
  reset_timer_phase: { args: { task_id: string }; ret: Timer }
  skip_phase: { args: { task_id: string }; ret: Timer }
  switch_active_task: {
    args: { task_id: string; old_task_id: string | null }
    ret: Timer
  }
  get_task: { args: { id: string }; ret: Task }
  get_active_task: { args: void; ret: Task | null }

  get_all_tasks: { args: void; ret: Task[] }
  create_task: { args: { request: CreateTaskRequest }; ret: Task }
  update_task: { args: { request: UpdateTaskRequest }; ret: Task }
  delete_task: { args: { id: string }; ret: void }
  complete_task: { args: { task_id: string }; ret: Task }
  reset_task: { args: { task_id: string }; ret: [Timer, Task] }
  get_global_config: { args: void; ret: Config }
  save_global_config: { args: { config: Config }; ret: void }
  reset_config_to_defaults: { args: void; ret: Config }
  test_audio_preview: { args: { asset_id: string; volume: number }; ret: PlaybackHandle }
  open_data_directory: { args: void; ret: void }
  clear_all_data: { args: void; ret: void }
  activate_screen_block: { args: void; ret: void }
  deactivate_screen_block: { args: void; ret: void }
}

/**
 * True once the webview has started tearing down (reload, navigation, or Vite
 * HMR module swap). While set, {@link invokeCmd} refuses to dispatch new IPC
 * calls so they cannot register callback ids that Rust will later be unable
 * to deliver to (the `[TAURI] Couldn't find callback id <n>` warning).
 *
 * NOTE: this cannot suppress the warning for calls that were ALREADY in flight
 * when the teardown began — those JS promises are gone, and Rust will still
 * log when it fails to find their callback. This only prevents the cascade of
 * NEW invokes fired by event listeners/effects during the teardown window.
 */
let isTearingDown = false

if (typeof window !== 'undefined') {
  for (const evtName of ['pagehide', 'beforeunload'] as const) {
    window.addEventListener(evtName, () => {
      isTearingDown = true
    })
  }
}

// Vite HMR: a module swap re-runs effects/listeners that may immediately
// fire invokes. Mark teardown during dispose so the new module knows to
// stay quiet until the page settles.
if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    isTearingDown = true
  })
}

/**
 * Type-safe wrapper around `@tauri-apps/api/core` `invoke`.
 * Rejects with {@link BackendError} on failure (never a raw string).
 *
 * During webview teardown (reload / HMR) this returns a forever-pending
 * promise instead of dispatching IPC, so no orphaned Rust callback is
 * registered. Callers with `.catch` handlers (e.g. fire-and-forget screen
 * blocker) are unaffected; they simply never hear back.
 */
export function invokeCmd<K extends keyof CommandMap>(
  command: K,
  ...rest: CommandMap[K]['args'] extends void ? [] : [CommandMap[K]['args']]
): Promise<CommandMap[K]['ret']> {
  if (isTearingDown) {
    return new Promise<CommandMap[K]['ret']>(() => {})
  }
  const args = rest[0] as Record<string, unknown> | undefined
  return invoke<CommandMap[K]['ret']>(command, args).catch((cause: unknown) => {
    throw new BackendError({ command, args, cause })
  })
}

/**
 * Single source of truth for Tauri event names.
 * These strings MUST match the `window.emit(...)` names in
 * apps/tauri-app (see the Emitter adapter). Drift = compile error here.
 */
export const events = {
  appInitialized: 'app:initialized',
  taskListUpdated: 'task:list_updated',
  taskActiveChanged: 'task:active_changed',
  taskCompleted: 'task:task_completed',
  taskReset: 'task:task_reset',
  taskProgressUpdated: 'task:progress_updated',
  taskAutoAdvanced: 'task:auto_advanced',
  timerTick: 'timer:tick',
  timerStatusChanged: 'timer:status_changed',
  timerPhaseCompleted: 'timer:phase_completed',
  timerPhaseSkipped: 'timer:phase_skipped',
  timerReset: 'timer:timer_reset',
  timerPaused: 'timer:timer_paused',
  timerStarted: 'timer:timer_started',
  timerResumed: 'timer:timer_resumed',
  screenBlockerActivate: 'screen_blocker:activate',
} as const

export type EventName = (typeof events)[keyof typeof events]

interface EventPayloadMap {
  'app:initialized': undefined
  'task:list_updated': unknown
  'task:active_changed': TaskActiveChangedPayload | null
  'task:task_completed': TaskCompletedPayload
  'task:progress_updated': Task
  'task:auto_advanced': TaskAutoAdvancedPayload
  'timer:tick': TickPayload
  'timer:status_changed': TimerStatusChangedPayload
  'timer:phase_completed': TimerStateData
  'timer:phase_skipped': PhaseSkippedPayload
  'timer:timer_reset': TimerStateData
  'timer:timer_started': TimerStateData
  'timer:timer_paused': TimerStateData
  'timer:timer_resumed': TimerStateData
  'task:task_reset': unknown
  'screen_blocker:activate': { message: string }
}

/**
 * Type-safe wrapper around `@tauri-apps/api/event` `listen`.
 * The handler receives the already-unwrapped event payload.
 */
export function onEvent<K extends keyof EventPayloadMap>(
  name: K,
  handler: (payload: EventPayloadMap[K]) => void,
): Promise<UnlistenFn> {
  return listen(name, (e) => handler(e.payload as EventPayloadMap[K]))
}

/**
 * Subscribes to every event in {@link events} and logs each payload to the
 * browser console. Returns a cleanup that unlistens all of them.
 * Dev aid for inspecting incoming Tauri events.
 */
export function logTauriEvents(): () => void {
  const unlisteners = Object.values(events).map((name) =>
    listen(name, (e) =>
      console.log(`%c[tauri ${name}]`, 'color:#6ab0ff;font-weight:bold', e.payload),
    ),
  )
  return () => {
    for (const p of unlisteners) void p.then((fn) => fn())
  }
}
