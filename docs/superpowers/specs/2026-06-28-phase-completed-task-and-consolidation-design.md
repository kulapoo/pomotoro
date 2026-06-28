# Phase-completed task payload + auto-advanced timer payload + useTasks consolidation

**Date:** 2026-06-28
**Status:** Approved (pending spec review)
**Predecessor:** `docs/superpowers/specs/2026-06-28-eventbus-direct-payload-mapping-design.md` (merged 2026-06-28 at `dae963a`)

## Motivation

The just-merged EventBus refactor direct-mapped task events to store state but left three gaps:

1. **`timer:phase_completed`** still carries only the `TimerStateData`. When a work session completes naturally, the bound task's `current_sessions` increments on the backend, but the frontend never learns the new task object — it must wait for the next `fetchTimer()` (which doesn't return a task) or for the user to navigate to the Tasks page.
2. **`task:auto_advanced`** carries the new task but not the new task's timer state. The frontend must call `fetchTimer()` after every auto-advance — exactly the IPC round-trp pattern the prior refactor was meant to eliminate.
3. **`useTasksEventBus`** (mounted only on `TasksPage`) duplicates two event subscriptions from the global `EventBus` (`task:completed`, `task:auto_advanced`), discards the `task:progress_updated` payload (a bare `Task` — instead triggers a full `get_active_task` IPC reload), and uses a 500 ms `setTimeout` + `createBatchedLoader` pattern that the prior refactor explicitly removed from `EventBus.ts`. The result is two divergent subscription strategies in the same codebase.

## Scope

**In scope:**
- Embed `task` in every `timer:phase_completed` emit (2 sites).
- Embed `timer` in every `task:auto_advanced` emit (3 sites).
- Subscribe to `task:progress_updated` in `EventBus.ts` (currently discarded).
- Move the `task:list_updated` IPC-reload trigger into `EventBus.ts`.
- Delete `useTasksEventBus` + its call site.

**Out of scope (follow-ups):**
- Standardizing `task:list_updated`'s payload shape (currently heterogeneous across 7 backend emitters — `Task`, `TaskCreated`, `TaskDeleted`, etc.). The event remains a "dirty list" signal that triggers a full `get_all_tasks` IPC reload.
- Embedding `timer` in `task:active_changed`, `task:completed`, `task:reset` payloads. Those three events still require `fetchTimer()` on the frontend.
- Adding `task` to other timer events (`tick`, `paused`, `resumed`, `started`, `reset`).

## Payload shape changes

### `timer:phase_completed`

| Aspect | Current | Proposed |
| --- | --- | --- |
| Shape | `TimerStateData` (bare) | `{ timer: TimerStateData, task: Task }` (envelope) |
| Rationale | Two domain objects (timer state + bound task); envelope keeps them distinct and avoids field-name collisions. |

Frontend type addition (`useTasks.ts`):

```ts
export interface PhaseCompletedPayload {
  timer: TimerStateData
  task: Task
}
```

`tauri.ts` `EventPayloadMap['timer:phase_completed']` changes from `TimerStateData` to `PhaseCompletedPayload`.

### `task:auto_advanced`

| Aspect | Current | Proposed |
| --- | --- | --- |
| Shape | `{ from_task_id, to_task_id, to_task }` | `{ from_task_id, to_task_id, to_task, timer: TimerStateData }` |
| Rationale | Additive — matches the established pattern from the prior refactor. |

`tauri.ts` `EventPayloadMap['task:auto_advanced']` is unchanged in structure; `TaskAutoAdvancedPayload` in `useTasks.ts` gains `timer: TimerStateData`.

## Backend emitter changes

### `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs` (3 edits)

All edits are inside the existing `match outcome` arms. No new fields, no constructor changes.

**Edit A — PHASE_COMPLETED envelope (Started arm, current L108-117):**

```rust
let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

self.emitter
    .emit(
        ui_listeners::timer::PHASE_COMPLETED,
        json!({ "timer": state_json, "task": task }),
    )
    .map_err(...)?;
```

`task` is already in scope (destructured from `PhaseOutcome::Started { task, .. }` at L86-92).

**Edit B — AUTO_ADVANCED adds timer (Started arm, current L119-134):**

```rust
if let Some(to_task_id) = cycled_to {
    self.emitter
        .emit(
            ui_listeners::task::AUTO_ADVANCED,
            json!({
                "from_task_id": countdown_expired.task_id,
                "to_task_id": to_task_id,
                "to_task": task,
                "timer": state_json,
            }),
        )
        .map_err(...)?;
}
```

`state_json` from Edit A is still in scope (same arm, no intervening rebind).

**Edit C — AUTO_ADVANCED adds timer (Paused arm, current L178-193):**

The Paused arm destructures `timer` from `PhaseOutcome::Paused { task, timer, .. }` at L158-164. Use it directly:

```rust
if let Some(to_task_id) = cycled_to {
    self.emitter
        .emit(
            ui_listeners::task::AUTO_ADVANCED,
            json!({
                "from_task_id": countdown_expired.task_id,
                "to_task_id": to_task_id,
                "to_task": task,
                "timer": timer.state(),
            }),
        )
        .map_err(...)?;
}
```

`timer.state()` is the freshly-loaded timer state because `load_state()` was called at L165 before this block.

### `core/infra/src/adapters/timer/event_handlers/phase_skipped.rs` (1 edit + wiring)

This handler currently has only `emitter` + `timer_srv`. We add `task_repository`.

**Struct + constructor:**

```rust
pub struct PhaseSkippedHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
}

impl PhaseSkippedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        Self { emitter, timer_srv, task_repository }
    }
}
```

**Handle body** — fetch the active task by ID read from timer state, embed in PHASE_COMPLETED envelope. Defensive: if task is missing or timer state has no `task_id`, emit `task: null` and log a warning (matches `TaskUpdatedHandler`'s missing-task policy from the prior refactor).

```rust
async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
    let _phase_skipped = event
        .as_any()
        .downcast_ref::<domain::PhaseSkipped>()
        .ok_or(domain::Error::EventHandlingError {
            message: "Failed to skip phase".to_string(),
        })?;

    self.timer_srv.load_state().await?;
    let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

    // Fetch the active task so PHASE_COMPLETED's payload shape matches the
    // natural-expiry path. Manual skips do not change task state, so this
    // is the same task that was bound before the skip.
    let task_json = match self.timer_srv.with_timer(|t| t.state().task_id).await {
        Some(task_id) => match self.task_repository.get_by_id(task_id).await {
            Ok(Some(task)) => json!(task),
            Ok(None) => {
                log::warn!(
                    "PhaseSkippedHandler: task {} not found; emitting task: null",
                    task_id
                );
                json!(null)
            }
            Err(e) => {
                log::warn!(
                    "PhaseSkippedHandler: failed to load task {}: {e}; emitting task: null",
                    task_id
                );
                json!(null)
            }
        },
        None => {
            log::warn!(
                "PhaseSkippedHandler: timer has no task_id; emitting task: null"
            );
            json!(null)
        },
    };

    let payload = json!({ "timer": state_json, "task": task_json });

    self.emitter
        .emit(domain::event_names::timer::PHASE_SKIPPED, payload.clone())
        .map_err(...)?;

    self.emitter
        .emit(domain::event_names::timer::PHASE_COMPLETED, payload)
        .map_err(...)?;

    Ok(())
}
```

> **Note on `state().task_id`:** This assumes `TimerStateData` exposes `task_id` as `Option<TaskId>`. Verify against `core/domain/src/timer/state.rs` during implementation. If the field shape differs, adapt accordingly.

### `core/infra/src/adapters/timer/event_handlers/registry.rs`

Update the `PhaseSkippedHandler::new()` call site (currently L32-35) to pass `task_repository.clone()`.

### `apps/tauri-app/src/commands/task_cmd/complete_flow.rs` (1 edit)

**Current L156-163:**

```rust
let _ = app_handle.emit(
    domain::event_names::task::AUTO_ADVANCED,
    json!({
        "from_task_id": task_id.to_string(),
        "to_task_id": plan.next_task_id.to_string(),
        "to_task": to_task,
    }),
);
```

**Proposed:**

```rust
let timer_json = timer_tick_service
    .with_timer(|t| json!(t.state()))
    .await;

let _ = app_handle.emit(
    domain::event_names::task::AUTO_ADVANCED,
    json!({
        "from_task_id": task_id.to_string(),
        "to_task_id": plan.next_task_id.to_string(),
        "to_task": to_task,
        "timer": timer_json,
    }),
);
```

Safe because `timer_tick_service.load_state()` was called at L90-93 before this block.

## Frontend changes

### `apps/react-ui/src/app/EventBus.ts`

**PHASE_COMPLETED handler** — unwrap the new envelope, apply both timer state and task:

```ts
onEvent(events.timerPhaseCompleted, (payload) => {
  applyTimerState(payload.timer)
  applyTaskIfActiveForId(payload.task.id, payload.task)
}),
```

Replaces the current `onEvent(events.timerPhaseCompleted, applyTimerState)`.

**AUTO_ADVANCED handler** — drop `fetchTimer()`, apply embedded timer:

```ts
onEvent(events.taskAutoAdvanced, (payload) => {
  applyActiveTask(payload.to_task)
  applyTimerState(payload.timer)
  toast.success('Switched to next task')
}),
```

**NEW subscription: `taskProgressUpdated`** — direct-map bare `Task`:

```ts
onEvent(events.taskProgressUpdated, (task) => {
  applyTaskIfActiveForId(task.id, task)
}),
```

Replaces the discarded-payload handler in `useTasksEventBus`. Fires on every natural work-session completion that doesn't cycle to a new task — the only signal for `current_sessions` increments.

**NEW subscription: `taskListUpdated`** — IPC reload of both `tasks[]` and `activeTask`, payload treated as opaque:

```ts
onEvent(events.taskListUpdated, () => {
  const s = useTaskStore.getState()
  void s.loadTasks()
  void s.loadActiveTask()
}),
```

`useTaskStore.getState()` (Zustand vanilla API) avoids adding the setters to the deps array and to the hook's selector list.

> **Why `loadActiveTask()` is retained here:** Some backend mutations emit only `task:list_updated` and no specific event the frontend listens to — notably `task_updated` (renames, config edits made on TasksPage). Without `loadActiveTask()`, editing the active task's name would leave `activeTask` stale until the next `fetchTimer()`. The prior `useTasksEventBus` did this reload; we preserve it.

**Effect deps array** — unchanged in shape (still `[fetchTimer, applyTick, applyTimerState, applyActiveTask, applyTaskIfActiveForId]`). The new `taskProgressUpdated` / `taskListUpdated` handlers reference stable store methods only.

### `apps/react-ui/src/pages/tasks/useTasks.ts`

Delete:
- `import { createBatchedLoader } from '@/lib/async'` (L7)
- The entire `useTasksEventBus` function (L244-272)

Keep:
- All payload-type interfaces (`TaskActiveChangedPayload`, `TaskCompletedPayload`, `TaskResetPayload`, `TaskAutoAdvancedPayload`).
- The new `PhaseCompletedPayload` interface (added under "Payload shape changes" above).
- `applyActiveTask`, `applyTaskIfActiveForId` setters (still consumed by `EventBus.ts`).

### `apps/react-ui/src/pages/tasks/TasksPage.tsx`

Remove `useTasksEventBus()` call (L25). The page now subscribes to no events directly — global `EventBus` covers everything.

## What stays the same

- **`fetchTimer()` retained on 3 task events:** `taskActiveChanged`, `taskCompleted`, `taskReset`. Their payloads do not carry timer state, so an explicit re-read is still required after active-task changes.
- **`task:list_updated` backend payload shape:** heterogeneous across 7 emitters; standardized refactoring tracked as a follow-up.
- **Setter signatures:** `applyTimerState`, `applyActiveTask`, `applyTaskIfActiveForId` unchanged.
- **Tick-loop ownership contract** (`CLAUDE.md`): unchanged — no handler in this spec mutates `cancel_handle`. All emit-only.

## Test strategy

### Backend (Rust)

Extend `core/infra/tests/app/task_event_payloads.rs` with:

1. **`phase_completed_payload_embeds_task_and_timer`** — publish `CountdownExpired(Work)`, capture the `timer:phase_completed` emit, assert both `payload["timer"]["state"]` (or whichever top-level key `TimerStateData` serializes) and `payload["task"]["id"]` are present and consistent.
2. **`auto_advanced_payload_embeds_timer_on_natural_cycle`** — drive a cycling scenario (`CountdownExpired(Work)` on a task with 1 remaining session that triggers auto-advance), assert `payload["timer"]` exists and matches the new task's timer state.
3. **`phase_skipped_payload_embeds_task`** — publish `PhaseSkipped`, assert `payload["task"]["id"]` matches the bound task. (New test — no existing coverage for `PhaseSkippedHandler`'s emit.)

Manual complete-flow path (`complete_flow.rs`) remains covered only by `cargo check -p tauri-app` and the smoke test (no integration test exists for Tauri command paths).

### Frontend

No JS test framework. Verify via:
- `npm run typecheck` — confirms payload type changes compile.
- `npm run lint`
- `npm run build`

### Manual smoke (deferred to human)

Run in dev Tauri:
1. Start work session, let it expire naturally → Tasks page shows incremented `current_sessions` without reload.
2. Manually skip phase (button on Timer page) → no console errors; payload shape consistent.
3. Configure `AutoAdvance`, exhaust a task, observe auto-advance → Timer page updates without IPC latency (no `fetchTimer` network call in DevTools).
4. Manually complete task with `AutoAdvance` from tray → same.
5. Create / edit / delete task on Tasks page → list reloads via `task:list_updated`.

## Risks

- **`TimerStateData::task_id` field shape:** The `phase_skipped.rs` edit assumes `state().task_id` returns `Option<TaskId>`. If the actual shape differs (e.g. `Option<String>`, or nested), the implementation must adapt. Mitigation: read `core/domain/src/timer/state.rs` before implementing.
- **`registry.rs` wiring:** Forgetting to pass `task_repository.clone()` to `PhaseSkippedHandler::new()` will fail at compile time (Rust's exhaustiveness).
- **Race in `phase_skipped.rs`:** If the active task is deleted between `load_state()` and `get_by_id()`, the handler emits `task: null` and the frontend guards via `applyTaskIfActiveForId` (no-op when payload task doesn't match). Acceptable.
- **Subscription ordering:** With `taskListUpdated` now global, every task change triggers a full reload even when the user is not on the Tasks page. This is the prior behavior (just relocated); no perf regression.

## Open questions

None. All design forks resolved during brainstorming:
- Envelope shape for `PHASE_COMPLETED` ✓
- Add `task_repository` to `PhaseSkippedHandler` ✓
- Full `useTasksEventBus` deletion ✓
