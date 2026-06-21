import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { BackendError } from '@/lib/errors'
import type { Timer, Phase } from '@/features/timer/types'
import type { Config } from '@/features/settings/types'
import type { Task, CreateTaskRequest, UpdateTaskRequest } from '@/features/tasks/types'

/**
 * Single source of truth for Tauri command names.
 * These strings MUST match the `#[tauri::command]` fn names registered in
 * apps/tauri-app/src/lib.rs (generate_handler!). Drift = compile error here.
 */
export const commands = {
  getTimerState: 'get_timer_state',
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
} as const

export type CommandName = (typeof commands)[keyof typeof commands]

interface CommandMap {
  get_timer_state: { args: void; ret: Timer }
  start_timer: { args: { task_id: string }; ret: void }
  pause_timer: { args: { task_id: string }; ret: void }
  resume_timer: { args: { task_id: string }; ret: void }
  reset_timer: { args: { task_id: string }; ret: void }
  reset_timer_phase: { args: { task_id: string }; ret: void }
  skip_phase: { args: { task_id: string }; ret: void }
  switch_active_task: { args: { task_id: string; old_task_id: string | null }; ret: void }
  get_all_tasks: { args: void; ret: Task[] }
  create_task: { args: { request: CreateTaskRequest }; ret: void }
  update_task: { args: { request: UpdateTaskRequest }; ret: void }
  delete_task: { args: { id: string }; ret: void }
  complete_task: { args: { task_id: string }; ret: void }
  reset_task: { args: { task_id: string }; ret: void }
  get_global_config: { args: void; ret: Config }
  save_global_config: { args: { config: Config }; ret: void }
  reset_config_to_defaults: { args: void; ret: void }
  test_audio_preview: { args: { sound_type: string }; ret: void }
  open_data_directory: { args: void; ret: void }
  clear_all_data: { args: void; ret: void }
}

/**
 * Type-safe wrapper around `@tauri-apps/api/core` `invoke`.
 * Rejects with {@link BackendError} on failure (never a raw string).
 */
export function invokeCmd<K extends keyof CommandMap>(
  command: K,
  ...rest: CommandMap[K]['args'] extends void ? [] : [CommandMap[K]['args']]
): Promise<CommandMap[K]['ret']> {
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
  taskProgressUpdated: 'task:progress_updated',
  taskAutoAdvanced: 'task:auto_advanced',
  timerTick: 'timer:tick',
  timerStatusChanged: 'timer:status_changed',
  timerPhaseCompleted: 'timer:phase_completed',
  timerPhaseSkipped: 'timer:phase_skipped',
  timerReset: 'timer:timer_reset',
  timerPaused: 'timer:timer_paused',
  timerResumed: 'timer:timer_resumed',
} as const

export type EventName = (typeof events)[keyof typeof events]

interface EventPayloadMap {
  'app:initialized': undefined
  'task:list_updated': undefined
  'task:active_changed': undefined
  'task:task_completed': undefined
  'task:progress_updated': undefined
  'task:auto_advanced': undefined
  'timer:tick': { task_id: string; phase: Phase; remaining_seconds: number }
  'timer:status_changed': undefined
  'timer:phase_completed': undefined
  'timer:phase_skipped': undefined
  'timer:timer_reset': undefined
  'timer:timer_paused': undefined
  'timer:timer_resumed': undefined
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
