# EventBus Direct Payload Mapping — Design

**Date:** 2026-06-28
**Status:** Approved (pending implementation plan)
**Related:** `docs/superpowers/specs/2026-06-27-task-switch-resets-timer-design.md` (timer reconciliation on task switch), `docs/superpowers/plans/2026-06-27-tick-loop-direct-drive.md` (tick-loop ownership contract)

## Goal

Replace IPC round-trips in `apps/react-ui/src/app/EventBus.ts` with direct state updates derived from event payloads. Each event handler should `set({ ... })` from its payload, not re-invoke a Tauri command.

## Problem

Today every meaningful event triggers `loadActiveTask()` / `fetchTimer()`, each of which is a full IPC `invoke` plus a `set`. A burst of related events collapses through `createBatchedLoader`, but each burst still pays one round-trip per store. Worse, `EventBus.ts` wraps several handlers in a `setTimeout(..., 500)` hack to give the backend time to commit before reading.

A second issue: the four task events (`task:active_changed`, `task:task_completed`, `task:task_reset`, `task:auto_advanced`) carry only IDs and audit fields — they cannot reconstruct a `Task` object on the client. The frontend is therefore *forced* to call `get_active_task` to learn what changed.

## Field-Mapping Audit

Per-event, can the payload fully populate its target store slice?

| Event                    | Payload                                          | Target                | Direct-mappable?                                           |
| ------------------------ | ------------------------------------------------ | --------------------- | ---------------------------------------------------------- |
| `timer:tick`               | `TickPayload`                                      | `timer.state.remaining_seconds` | Yes — already direct via `applyTick` |
| `timer:phase_completed`    | `TimerStateData`                                   | `timer.state`          | Yes — all `state` fields present (`state`, `remaining_seconds?`, `paused_from?`) |
| `timer:timer_reset`        | `TimerStateData`                                   | `timer.state`          | Yes |
| `timer:timer_started`      | `TimerStateData`                                   | `timer.state`          | Yes |
| `timer:timer_paused`       | `TimerStateData`                                   | `timer.state`          | Yes |
| `timer:timer_resumed`      | `TimerStateData`                                   | `timer.state`          | Yes |
| `task:active_changed`      | IDs + audit only                                   | `activeTask: Task`     | **No** — needs backend patch |
| `task:task_completed`      | IDs + audit only                                   | `activeTask: Task`     | **No** — needs backend patch |
| `task:task_reset`          | IDs + audit only                                   | `activeTask: Task`     | **No** — needs backend patch |
| `task:auto_advanced`       | `from_task_id` + `to_task_id`                       | `activeTask: Task`     | **No** — needs backend patch |
| `screen_blocker:activate`  | `{ message }`                                       | `screenBlocker`        | Yes — already direct |

Two caveats on the timer rows:

1. **`task_id` is not in `TimerStateData`.** `Timer = { task_id, state }`. Direct map preserves the existing `task_id` from the store. This is correct because `task_id` only mutates via a separate task event (`task:active_changed`), after which the orchestrator's tick loop restart will emit a fresh `timer:*` event.
2. **`timer:phase_completed` currently drives a 500 ms-delayed full reload.** That reload exists to reconcile side effects (session counts, auto-advance, screen blocker). After this refactor, those side effects are reconciled by their own events (`task:auto_advanced`, `task:task_completed`, `screen_blocker:activate`), so the reload is no longer needed for the timer slice itself.

## Chosen Design

### Backend (Rust): embed full `Task` in task-event payloads

Mirror the existing `TaskUpdatedHandler` pattern (`core/infra/src/adapters/task/event_handlers/task_updated.rs:34`): handler takes `Arc<dyn TaskRepository + Send + Sync>`, fetches the relevant task, embeds it in the emitted JSON. Payload shape is **additive** — existing audit fields are kept; new `task` / `to_task` field is added. No existing consumer breaks (the tray only listens to event names, not fields).

**Three domain-event handlers** (`core/infra/src/adapters/task/event_handlers/`):

| Handler                    | Subscribes to          | Fetch by         | New payload key |
| -------------------------- | ---------------------- | ---------------- | --------------- |
| `TaskActiveChangedHandler` | `domain::ActiveChanged` | `new_task_id`    | `task: Task`    |
| `TaskCompletedHandler`     | `domain::Completed`     | `task_id`        | `task: Task`    |
| `TaskResetHandler`         | `domain::Reset`         | `task_id`        | `task: Task`    |

The registry (`registry.rs:31-32`) already has `task_repository` in scope and passes it to `TaskUpdatedHandler`; thread it into the three handlers above.

If the repo returns `None` (task vanished between commit and emit), log a warning and skip the emit — same behavior as `TaskUpdatedHandler`.

**Three inline `task:auto_advanced` emit sites:**

| File                                | Line | `to_task` source |
| ----------------------------------- | ---- | ---------------- |
| `countdown_expired.rs` (Started)    | 122  | `task` already in scope (from `PhaseOutcome::Started { task, .. }`) |
| `countdown_expired.rs` (Paused)     | 180  | `task` already in scope (from `PhaseOutcome::Paused { task, .. }`) |
| `complete_flow.rs`                  | 146  | **needs a fetch** — `next_task` on line 117 is bound inside an inner `if plan.auto_start_work` block and is not in scope at the emit site. Add a `task_repo.get_by_id(plan.next_task_id)` call immediately before the emit. The task was just committed by the usecase, so the read is consistent. |

Emit `json!({ "from_task_id": ..., "to_task_id": ..., "to_task": to_task })`.

### Frontend: direct setters, no `setTimeout`

**`apps/react-ui/src/lib/tauri.ts`** — update payload types:

```ts
'task:active_changed': TaskActiveChangedPayload   // gains `task: Task`
'task:task_completed': TaskCompletedPayload       // gains `task: Task`
'task:task_reset':    TaskResetPayload            // new typed shape with `task: Task`
'task:auto_advanced': TaskAutoAdvancedPayload     // gains `to_task: Task`
```

**`apps/react-ui/src/pages/tasks/useTasks.ts`** — add two store actions:

- `applyActiveTask(task: Task)` — unconditional `set({ activeTask: task })`. Used by `task:active_changed` and `task:auto_advanced`.
- `applyTaskIfActiveForId(taskId: string, task: Task)` — only sets if `taskId === get().activeTask?.id`. Used by `task:task_completed` and `task:task_reset`, which may fire for non-active tasks.

**`apps/react-ui/src/pages/timer/useTimer.ts`** — add `applyTimerState(state: TimerStateData)` that preserves the existing `task_id`:

```ts
applyTimerState: (state) => set((s) => ({
  timer: s.timer ? { task_id: s.timer.task_id, state } : s.timer,
}))
```

**`apps/react-ui/src/app/EventBus.ts`** — rewrite handlers:

| Event                    | Handler                                                          |
| ------------------------ | ---------------------------------------------------------------- |
| `timer:tick`               | `applyTick(payload)` (unchanged)                                  |
| `timer:phase_completed`    | `applyTimerState(payload)` — drop the 500 ms reload              |
| `timer:timer_reset`        | `applyTimerState(payload)`                                       |
| `timer:timer_started`      | `applyTimerState(payload)`                                       |
| `timer:timer_paused`       | `applyTimerState(payload)`                                       |
| `timer:timer_resumed`      | `applyTimerState(payload)`                                       |
| `task:active_changed`      | `applyActiveTask(payload.task)` + `fetchTimer()` (no delay)      |
| `task:task_completed`      | `applyTaskIfActiveForId(payload.task_id, payload.task)` + `fetchTimer()` + toast |
| `task:task_reset`          | `applyTaskIfActiveForId(payload.task_id, payload.task)` + `fetchTimer()` + toast |
| `task:auto_advanced`       | `applyActiveTask(payload.to_task)` + `fetchTimer()` + toast      |
| `screen_blocker:activate`  | `useScreenBlockerStore.getState().activate(payload.message)` (unchanged) |

`fetchTimer` is kept on task events: orchestrators (`switch_active_task`, `reset_task`, etc.) do **not** emit `timer:*` events after `load_state` (documented gap, `2026-06-27-task-switch-resets-timer-design.md:97`). Non-calling windows therefore need an explicit timer fetch to stay in sync with the new task's timer state. The 500 ms delay is dropped because the active-task slice is now derived from the event payload (no commit-race).

`createBatchedLoader` is removed from `EventBus.ts` — no more batched fetches to coalesce. (`useTasksEventBus` in `useTasks.ts` retains its own batched loader for the Tasks page; that is out of scope.)

### Task type re-export

`Task` is already exported from `useTasks.ts` and imported into `tauri.ts`. No new exports needed.

## Out of Scope

- Emitting `timer:*` events from the `switch_active_task` / `reset_task` / `complete_flow` orchestrators. That would eliminate the remaining `fetchTimer` calls but expands the Rust surface into the orchestrator layer; deferred to a follow-up.
- Refactoring `useTasksEventBus` (the Tasks-page-scoped bus in `useTasks.ts`). It still uses `createBatchedLoader` + `loadTasks` for the full task list. Separate concern.
- The `task:list_updated` event (broadcast, not active-task-specific). Left untouched.

## Testing

- **Rust:** existing tests in `core/infra/tests/app/` cover the handlers; add assertions that the new `task` / `to_task` field is present and matches the repo state. Manually verify the auto-advance path in `manual_complete_cycling.rs`-style tests.
- **Frontend:** unit-test `applyActiveTask`, `applyTaskIfActiveForId`, `applyTimerState` for: (a) basic set, (b) `taskId` mismatch leaves store untouched, (c) `applyTimerState` preserves `task_id` when timer is non-null and no-ops when null.
- **Manual smoke test:** start a session, switch tasks from the tray (non-calling window), complete a task, reset a task — verify the Timer page reconciles without the 500 ms lag.

## Risks

- **Repo miss in handler.** If the task is deleted between the usecase commit and the handler's `get_by_id`, the emit is skipped. The frontend's `fetchTimer` fallback (kept on task events) is a safety net for the active-task slice; the toast/UX may momentarily lag. Acceptable.
- **Field-name drift.** The frontend `Task` interface (`useTasks.ts:19`) and the Rust `Task` struct (`core/domain/src/task/task.rs:8`) must keep field names in lock-step. Already true today; this refactor doesn't change either struct, just embeds them.
- **`applyTimerState` vs `applyTick` overlap.** A `timer:tick` arriving after a `timer:phase_completed` will overwrite `remaining_seconds` with the tick's value. This is correct (tick is the live source of truth) and matches today's behavior.
