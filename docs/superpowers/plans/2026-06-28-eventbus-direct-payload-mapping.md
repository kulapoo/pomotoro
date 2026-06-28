# EventBus Direct Payload Mapping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace IPC round-trips in `apps/react-ui/src/app/EventBus.ts` with direct state updates derived from event payloads, by embedding full `Task` objects in the four task-event payloads emitted from Rust.

**Architecture:** Backend task-event emitters mirror the existing `TaskUpdatedHandler` pattern (fetch task from repo, embed in JSON payload). Frontend adds three small store setters (`applyActiveTask`, `applyTaskIfActiveForId`, `applyTimerState`) and `EventBus.ts` becomes a pure payload→state mapper. `fetchTimer` is retained on the four task events because orchestrators don't emit `timer:*` events after `load_state` (documented gap — see spec).

**Tech Stack:** Rust (workspace member `core/infra`, `apps/tauri-app`), React 19 + Zustand 5 + TypeScript 5 + Vite 6 in `apps/react-ui`.

## Global Constraints

- **No frontend test framework.** `apps/react-ui/package.json` has no vitest/jest. Frontend tasks verify via `npm run typecheck` + `npm run lint` (run from `apps/react-ui/`). Do NOT add a test framework in this plan.
- **Backend test command:** `cargo test -p infra <test_name>` from workspace root. All new Rust tests live in `core/infra/tests/app/`.
- **Commit style:** Conventional Commits with scope, e.g. `feat(infra/task): embed Task in TaskCompletedHandler payload`. Match recent history (`git log --oneline -10`).
- **Field naming:** Backend `Task` already serializes to snake_case matching the frontend `Task` interface (`id`, `name`, `max_sessions`, `current_sessions`, `tags`, `config`, `created_at`, `updated_at`, `completed_at`, `status`). Do NOT change either struct.
- **Additive payloads only.** Keep all existing fields (`version`, `occurred_at`, `task_id`, etc.); add `task` / `to_task`. Don't break existing consumers (tray listens for event names, not fields).
- **Tick-loop ownership contract** (per `CLAUDE.md`): handlers are UI-only emitters and MUST NOT touch `TimerTickService::cancel_handle`. The handler changes here only *read* `TaskRepository` — no tick-loop mutation.

## File Structure

**Backend (modified):**
- `core/infra/src/adapters/task/event_handlers/task_completed.rs` — fetch + embed `task`
- `core/infra/src/adapters/task/event_handlers/task_active_changed.rs` — fetch + embed `task`
- `core/infra/src/adapters/task/event_handlers/task_reset.rs` — fetch + embed `task`
- `core/infra/src/adapters/task/event_handlers/registry.rs` — thread `task_repository` into the three handlers above (already receives it as a param)
- `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs` — embed `to_task` at lines 122 and 180
- `apps/tauri-app/src/commands/task_cmd/complete_flow.rs` — fetch `next_task` at the emit site (line 146), embed `to_task`

**Backend (new tests):**
- `core/infra/tests/app/task_event_payloads.rs` — integration tests asserting the four events now carry full task data
- `core/infra/tests/app/mod.rs` — add `mod task_event_payloads;`

**Frontend (modified):**
- `apps/react-ui/src/lib/tauri.ts` — extend payload types for the 4 task events
- `apps/react-ui/src/pages/tasks/useTasks.ts` — add `TaskResetPayload` type, `applyActiveTask`, `applyTaskIfActiveForId`
- `apps/react-ui/src/pages/timer/useTimer.ts` — add `applyTimerState`
- `apps/react-ui/src/app/EventBus.ts` — rewrite to direct-map

---

### Task 1: `TaskCompletedHandler` emits full `Task` payload

**Files:**
- Modify: `core/infra/src/adapters/task/event_handlers/task_completed.rs`
- Modify: `core/infra/src/adapters/task/event_handlers/registry.rs:21-22`
- Create: `core/infra/tests/app/task_event_payloads.rs`
- Modify: `core/infra/tests/app/mod.rs`

**Interfaces:**
- Produces: `task:task_completed` payload now contains `{ ..., task: <serialized domain::Task> }` alongside the existing `task_id`/`total_sessions`/`completed_at`/`version`/`occurred_at` fields.

- [ ] **Step 1: Write the failing test**

Create `core/infra/tests/app/task_event_payloads.rs`:

```rust
use std::time::Duration;

use domain::{Config, Phase, TaskStatus, event_names};
use usecases::task::complete_task;

use crate::{
    TaskBuilder,
    utils::{assert_utils, setup::setup_ctx},
};

/// `task:task_completed` payload must carry the full Task object (not just
/// audit fields) so the React EventBus can direct-map it into the
/// `activeTask` store slice without an IPC round-trip.
#[tokio::test]
async fn task_completed_payload_embeds_full_task() {
    let ctx =
        setup_ctx("task_completed_payload_embeds_full_task").await;

    let task = TaskBuilder::new()
        .name("Embed me")
        .max_sessions(3)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    let task_id = task.id();
    let expected_name = task.name().to_string();
    let expected_max = task.max_sessions();
    ctx.task_repo.create(task).await.unwrap();

    // `complete_task` signature: `(task_repo: &Arc<...>, event_publisher:
    // &Arc<...>, task_id: TaskId)`. See core/usecases/src/task/complete_task.rs.
    complete_task(&ctx.task_repo, &ctx.event_bus, task_id)
        .await
        .expect("complete_task failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::TASK_COMPLETED);
    assert!(
        !events.is_empty(),
        "task:task_completed was not emitted"
    );

    let payload = &events[0].payload;
    let embedded = payload
        .get("task")
        .expect("payload missing `task` field");
    assert_eq!(embedded["name"], expected_name);
    assert_eq!(embedded["max_sessions"], expected_max);
    assert_eq!(embedded["status"], "Completed");
    assert_eq!(
        embedded["id"],
        task_id.to_string(),
        "embedded task id must match the completed task"
    );
}
```

Add to `core/infra/tests/app/mod.rs` (after the existing `mod task_cycling;` line or similar):

```rust
mod task_event_payloads;
```

Check existing imports in `core/infra/tests/app/mod.rs` to confirm `TaskBuilder` re-export and `utils` path. The `complete_task` import path is `usecases::task::complete_task` (verified at `core/usecases/src/task/complete_task.rs:6`).

- [ ] **Step 2: Run test to verify it fails**

```
cargo test -p infra task_completed_payload_embeds_full_task -- --nocapture
```

Expected: FAIL with `payload missing `task` field` (panicked at the `.expect`).

- [ ] **Step 3: Patch the handler**

Modify `core/infra/src/adapters/task/event_handlers/task_completed.rs`:

Replace top imports:
```rust
use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;
```

Replace struct + impl:
```rust
pub struct TaskCompletedHandler {
    emitter: Arc<dyn Emitter>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TaskCompletedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repo: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        TaskCompletedHandler { emitter, task_repo }
    }
}
```

Replace `handle` body (both emits) with a single combined payload:
```rust
    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_completed = event
            .as_any()
            .downcast_ref::<domain::TaskCompleted>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to complete task".to_string(),
            })?;

        let task = self
            .task_repo
            .get_by_id(task_completed.task_id)
            .await?;

        let Some(task) = task else {
            log::warn!(
                "Task {} not found after completion; skipping emit",
                task_completed.task_id
            );
            return Ok(());
        };

        let payload = json!({
            "task_id": task_completed.task_id,
            "total_sessions": task_completed.total_sessions,
            "completed_at": task_completed.completed_at,
            "version": task_completed.version,
            "occurred_at": task_completed.occurred_at,
            "task": task,
        });

        self.emitter
            .emit(domain::event_names::task::LIST_UPDATED, payload.clone())
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task completed event: {e}"
                ),
            })?;

        self.emitter
            .emit(domain::event_names::task::TASK_COMPLETED, payload)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task completed event: {e}"
                ),
            })?;

        Ok(())
    }
```

Note: the existing handler emitted the bare event struct via `json!(task_completed)`. The new version builds the payload explicitly so `task` is included. `Id` serializes to a string (it's a `Uuid` newtype), matching the frontend's `id: string`.

- [ ] **Step 4: Update the registry call site**

Modify `core/infra/src/adapters/task/event_handlers/registry.rs:21-22`:

Replace:
```rust
    event_bus
        .subscribe(Box::new(TaskCompletedHandler::new(emitter.clone())))?;
```
with:
```rust
    event_bus.subscribe(Box::new(TaskCompletedHandler::new(
        emitter.clone(),
        task_repository.clone(),
    )))?;
```

- [ ] **Step 5: Run test to verify it passes**

```
cargo test -p infra task_completed_payload_embeds_full_task -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Run full infra test suite to confirm no regression**

```
cargo test -p infra
```

Expected: all tests pass. If `manual_complete_emits_task_completed_event` or any other existing test fails, inspect the assertion — most likely it only checks for event presence (still passes).

- [ ] **Step 7: Commit**

```bash
git add core/infra/src/adapters/task/event_handlers/task_completed.rs \
        core/infra/src/adapters/task/event_handlers/registry.rs \
        core/infra/tests/app/task_event_payloads.rs \
        core/infra/tests/app/mod.rs
git commit -m "feat(infra/task): embed full Task in TaskCompletedHandler payload"
```

---

### Task 2: `TaskActiveChangedHandler` emits full `Task` payload

**Files:**
- Modify: `core/infra/src/adapters/task/event_handlers/task_active_changed.rs`
- Modify: `core/infra/src/adapters/task/event_handlers/registry.rs:30-31`
- Modify: `core/infra/tests/app/task_event_payloads.rs` (append test)

**Interfaces:**
- Produces: `task:active_changed` payload now contains `{ ..., task: <serialized domain::Task> }` (the *new* active task) alongside existing `old_task_id`/`new_task_id`/`workflow_result`/`version`/`occurred_at`.

- [ ] **Step 1: Append the failing test**

Append to `core/infra/tests/app/task_event_payloads.rs`:

```rust
/// `task:active_changed` payload must carry the full new active Task so the
/// React EventBus can `set({ activeTask: payload.task })` directly.
#[tokio::test]
async fn active_changed_payload_embeds_full_task() {
    let ctx =
        setup_ctx("active_changed_payload_embeds_full_task").await;

    let task1 = TaskBuilder::new()
        .name("First")
        .max_sessions(2)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    ctx.task_repo.create(task1).await.unwrap();

    let task2 = TaskBuilder::new()
        .name("Second")
        .max_sessions(4)
        .status(TaskStatus::Queued)
        .config(Config::default())
        .build();
    let task2_id = task2.id();
    let task2_name = task2.name().to_string();
    ctx.task_repo.create(task2).await.unwrap();

    // Mark task1 as the current active by binding it to the timer (the
    // switch_active_task usecase reads the prior active from the timer).
    // Then switch to task2.
    use usecases::task::{
        SwitchActiveTaskCmd, switch_active_task,
    };
    switch_active_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        SwitchActiveTaskCmd {
            task_id: task2_id,
            old_task_id: None,
        },
    )
    .await
    .expect("switch_active_task failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::ACTIVE_CHANGED);
    assert!(
        !events.is_empty(),
        "task:active_changed was not emitted"
    );

    let payload = &events[0].payload;
    let embedded = payload
        .get("task")
        .expect("payload missing `task` field");
    assert_eq!(embedded["id"], task2_id.to_string());
    assert_eq!(embedded["name"], task2_name);
    assert_eq!(embedded["max_sessions"], 4);
}
```

- [ ] **Step 2: Run test to verify it fails**

```
cargo test -p infra active_changed_payload_embeds_full_task -- --nocapture
```

Expected: FAIL with `payload missing `task` field`.

If the test fails for a different reason (e.g. `switch_active_task` signature mismatch), adjust the import path. Grep first: `rg "pub async fn switch_active_task" core/usecases/src`.

- [ ] **Step 3: Patch the handler**

Modify `core/infra/src/adapters/task/event_handlers/task_active_changed.rs`:

Replace top imports:
```rust
use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskActiveChangedHandler {
    emitter: Arc<dyn Emitter>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TaskActiveChangedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repo: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        TaskActiveChangedHandler { emitter, task_repo }
    }
}
```

Replace the `handle` body:
```rust
    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_switch =
            event.as_any().downcast_ref::<domain::TaskActiveChanged>();

        let Some(switch) = task_switch else {
            return Ok(());
        };

        let task = self
            .task_repo
            .get_by_id(switch.new_task_id)
            .await?;

        let Some(task) = task else {
            log::warn!(
                "Task {} not found after active change; skipping emit",
                switch.new_task_id
            );
            return Ok(());
        };

        let payload = json!({
            "old_task_id": switch.old_task_id,
            "new_task_id": switch.new_task_id,
            "workflow_result": switch.workflow_result,
            "version": switch.version,
            "occurred_at": switch.occurred_at,
            "task": task,
        });

        self.emitter
            .emit(
                domain::event_names::task::ACTIVE_CHANGED,
                payload,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task active changed event: {e}"
                ),
            })?;
        Ok(())
    }
```

Note: existing handler emits `json!(task_switch)` directly. We construct the payload explicitly so we can attach the `task` field. Field names match the frontend `TaskActiveChangedPayload` (camelCase on read is via the TS interface; serde emits snake_case which already matches the existing TS shape — see `useTasks.ts:56-62`).

- [ ] **Step 4: Update the registry call site**

Modify `core/infra/src/adapters/task/event_handlers/registry.rs:30-31`:

Replace:
```rust
    event_bus
        .subscribe(Box::new(TaskActiveChangedHandler::new(emitter.clone())))?;
```
with:
```rust
    event_bus.subscribe(Box::new(TaskActiveChangedHandler::new(
        emitter.clone(),
        task_repository.clone(),
    )))?;
```

- [ ] **Step 5: Run test to verify it passes**

```
cargo test -p infra active_changed_payload_embeds_full_task -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Run full infra test suite**

```
cargo test -p infra
```

Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add core/infra/src/adapters/task/event_handlers/task_active_changed.rs \
        core/infra/src/adapters/task/event_handlers/registry.rs \
        core/infra/tests/app/task_event_payloads.rs
git commit -m "feat(infra/task): embed full Task in TaskActiveChangedHandler payload"
```

---

### Task 3: `TaskResetHandler` emits full `Task` payload

**Files:**
- Modify: `core/infra/src/adapters/task/event_handlers/task_reset.rs`
- Modify: `core/infra/src/adapters/task/event_handlers/registry.rs:32`
- Modify: `core/infra/tests/app/task_event_payloads.rs` (append test)

**Interfaces:**
- Produces: `task:task_reset` payload now contains `{ ..., task: <serialized domain::Task> }` (the reset task) alongside existing fields.

- [ ] **Step 1: Append the failing test**

Append to `core/infra/tests/app/task_event_payloads.rs`:

```rust
/// `task:task_reset` payload must carry the full reset Task so the React
/// EventBus can reconcile `activeTask` directly (when the reset task is the
/// active one).
#[tokio::test]
async fn task_reset_payload_embeds_full_task() {
    let ctx =
        setup_ctx("task_reset_payload_embeds_full_task").await;

    let task = TaskBuilder::new()
        .name("Reset me")
        .max_sessions(3)
        .status(TaskStatus::Active)
        .current_sessions(2)
        .config(Config::default())
        .build();
    let task_id = task.id();
    let task_name = task.name().to_string();
    ctx.task_repo.create(task).await.unwrap();

    use usecases::task::reset_task;
    reset_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        task_id,
    )
    .await
    .expect("reset_task failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::TASK_RESET);
    assert!(
        !events.is_empty(),
        "task:task_reset was not emitted"
    );

    let payload = &events[0].payload;
    let embedded = payload
        .get("task")
        .expect("payload missing `task` field");
    assert_eq!(embedded["id"], task_id.to_string());
    assert_eq!(embedded["name"], task_name);
    assert_eq!(embedded["current_sessions"], 0);
}
```

`reset_task` signature: `(task_repo, timer_repo, event_publisher, task_id: TaskId) -> Result<(Task, Timer)>`. Verified at `core/usecases/src/task/reset_task.rs:8`. Note it takes `timer_repo` as the 2nd positional arg — the `TaskReset` domain event is published *after* the timer reset, so by the time our handler reads the task from the repo, the reset has been committed.

- [ ] **Step 2: Run test to verify it fails**

```
cargo test -p infra task_reset_payload_embeds_full_task -- --nocapture
```

Expected: FAIL with `payload missing `task` field`.

- [ ] **Step 3: Patch the handler**

Modify `core/infra/src/adapters/task/event_handlers/task_reset.rs`:

Replace top imports:
```rust
use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TaskReset`.
///
/// Per the tick-loop ownership contract, this handler MUST NOT stop the tick
/// loop. The orchestrator that called `reset_task` owns the
/// `stop_timer_tick_loop` + `load_state` side effects.
pub struct TaskResetHandler {
    emitter: Arc<dyn Emitter>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TaskResetHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repo: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        TaskResetHandler { emitter, task_repo }
    }
}
```

Replace the `handle` body:
```rust
    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_reset = event
            .as_any()
            .downcast_ref::<domain::TaskReset>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to reset task".to_string(),
            })?;

        let task = self.task_repo.get_by_id(task_reset.task_id).await?;

        let Some(task) = task else {
            log::warn!(
                "Task {} not found after reset; skipping emit",
                task_reset.task_id
            );
            return Ok(());
        };

        let payload = json!({
            "task_id": task_reset.task_id,
            "name": task_reset.name,
            "description": task_reset.description,
            "max_sessions": task_reset.max_sessions,
            "tags": task_reset.tags,
            "version": task_reset.version,
            "occurred_at": task_reset.occurred_at,
            "task": task,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::TASK_RESET,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task reset event: {e}"
                ),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::LIST_UPDATED,
                payload,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task list updated event: {e}"
                ),
            })?;
        Ok(())
    }
```

- [ ] **Step 4: Update the registry call site**

Modify `core/infra/src/adapters/task/event_handlers/registry.rs:32`:

Replace:
```rust
    event_bus.subscribe(Box::new(TaskResetHandler::new(emitter.clone())))?;
```
with:
```rust
    event_bus.subscribe(Box::new(TaskResetHandler::new(
        emitter.clone(),
        task_repository.clone(),
    )))?;
```

- [ ] **Step 5: Run test to verify it passes**

```
cargo test -p infra task_reset_payload_embeds_full_task -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Run full infra test suite**

```
cargo test -p infra
```

Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add core/infra/src/adapters/task/event_handlers/task_reset.rs \
        core/infra/src/adapters/task/event_handlers/registry.rs \
        core/infra/tests/app/task_event_payloads.rs
git commit -m "feat(infra/task): embed full Task in TaskResetHandler payload"
```

---

### Task 4: `task:auto_advanced` carries `to_task` (both `countdown_expired.rs` arms + `complete_flow.rs`)

**Files:**
- Modify: `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs:119-133` (Started arm)
- Modify: `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs:177-191` (Paused arm)
- Modify: `apps/tauri-app/src/commands/task_cmd/complete_flow.rs:145-151` (needs new fetch)
- Modify: `core/infra/tests/app/task_event_payloads.rs` (append test)

**Interfaces:**
- Produces: all three `task:auto_advanced` emit sites now send `{ from_task_id, to_task_id, to_task: <serialized domain::Task> }`.

- [ ] **Step 1: Append the failing test**

Append to `core/infra/tests/app/task_event_payloads.rs`:

```rust
/// `task:auto_advanced` payload must carry the full next Task so the React
/// EventBus can `set({ activeTask: payload.to_task })` directly. Verified
/// via the CountdownExpiredHandler path (the cycle source for auto-advance).
#[tokio::test]
async fn auto_advanced_payload_embeds_to_task() {
    use domain::{TaskCyclingBehavior, timer::events::CountdownExpired};
    use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

    let ctx = AppContextBuilder::new()
        .with_name("auto_advanced_payload_embeds_to_task")
        .build()
        .await
        .expect("build ctx");

    // AutoAdvance + auto_start_work_after_break so the cycle fires. Field
    // names verified at core/domain/src/config/general.rs:11-14 — all on
    // `config.general` (no `cycling` sub-struct).
    let mut cfg = Config::default();
    cfg.general.task_cycling_behavior = TaskCyclingBehavior::AutoAdvance;
    cfg.general.auto_start_breaks = true;
    cfg.general.auto_start_work_after_break = true;
    ctx.config_repo.save_config(&cfg).await.unwrap();

    let task1 = TaskBuilder::new()
        .name("First")
        .max_sessions(1)
        .status(TaskStatus::Active)
        .config(cfg.clone())
        .build();
    let task1_id = task1.id();
    ctx.task_repo.create(task1).await.unwrap();

    let task2 = TaskBuilder::new()
        .name("Second")
        .max_sessions(2)
        .status(TaskStatus::Active)
        .config(cfg.clone())
        .build();
    let task2_id = task2.id();
    let task2_name = task2.name().to_string();
    ctx.task_repo.create(task2).await.unwrap();

    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd { task_id: Some(task1_id) },
    )
    .await
    .expect("start");

    tokio::time::sleep(Duration::from_millis(100)).await;
    ctx.ui_simulator.app_handle().clear_events();

    // Work phase completes the only session on task1 → cycled to task2.
    ctx.event_bus
        .publish(Box::new(CountdownExpired::new(Phase::Work, task1_id)));

    tokio::time::sleep(Duration::from_millis(300)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::AUTO_ADVANCED);
    assert!(
        !events.is_empty(),
        "task:auto_advanced was not emitted"
    );

    let payload = &events[0].payload;
    let embedded = payload
        .get("to_task")
        .expect("payload missing `to_task` field");
    assert_eq!(embedded["id"], task2_id.to_string());
    assert_eq!(embedded["name"], task2_name);
}
```

Update the file-level `use crate::{...}` import at the top of `task_event_payloads.rs` to also bring `AppContextBuilder` into scope (Task 1 set up `TaskBuilder` + `setup::setup_ctx` only):

```rust
use crate::{
    AppContextBuilder, TaskBuilder,
    utils::{assert_utils, setup::setup_ctx},
};
```

`Phase` is already imported via Task 1's top-of-file `use domain::{Config, Phase, TaskStatus, event_names};`. `TaskCyclingBehavior` enters scope via the per-test `use domain::{TaskCyclingBehavior, timer::events::CountdownExpired};` line inside the test body.

- [ ] **Step 2: Run test to verify it fails**

```
cargo test -p infra auto_advanced_payload_embeds_to_task -- --nocapture
```

Expected: FAIL with `payload missing `to_task` field`.

- [ ] **Step 3: Patch `countdown_expired.rs` Started arm (line 119-133)**

Replace the `if let Some(to_task_id) = cycled_to { ... }` block in the `Started` arm with:

```rust
                if let Some(to_task_id) = cycled_to {
                    self.emitter
                        .emit(
                            ui_listeners::task::AUTO_ADVANCED,
                            json!({
                                "from_task_id": countdown_expired.task_id,
                                "to_task_id": to_task_id,
                                "to_task": task,
                            }),
                        )
                        .map_err(|e| domain::Error::EventPublishingError {
                            message: format!(
                                "Failed to emit auto-advanced event: {e}"
                            ),
                        })?;
                }
```

Rationale: `task` is destructured from `PhaseOutcome::Started { task, .. }` on line 87 and is the *current* (post-cycle) task. Verified by the existing `PROGRESS_UPDATED` emit at line 136 which uses the same `task` variable for the cycled task's progress.

- [ ] **Step 4: Patch `countdown_expired.rs` Paused arm (line 177-191)**

Replace the `if let Some(to_task_id) = cycled_to { ... }` block in the `Paused` arm with the same shape (the `task` binding on line 159 is the cycled task):

```rust
                if let Some(to_task_id) = cycled_to {
                    self.emitter
                        .emit(
                            ui_listeners::task::AUTO_ADVANCED,
                            json!({
                                "from_task_id": countdown_expired.task_id,
                                "to_task_id": to_task_id,
                                "to_task": task,
                            }),
                        )
                        .map_err(|e| domain::Error::EventPublishingError {
                            message: format!(
                                "Failed to emit auto-advanced event: {e}"
                            ),
                        })?;
                }
```

- [ ] **Step 5: Patch `complete_flow.rs` (line 145-151)**

This emit site is at the end of the auto-advance branch (after the `if plan.auto_start_work` block). `next_task` is bound inside the inner block and is NOT in scope here — verified at `apps/tauri-app/src/commands/task_cmd/complete_flow.rs:117-141`.

Replace:
```rust
                let _ = app_handle.emit(
                    domain::event_names::task::AUTO_ADVANCED,
                    json!({
                        "from_task_id": task_id.to_string(),
                        "to_task_id": plan.next_task_id.to_string(),
                    }),
                );
```

with:
```rust
                let to_task = task_repo
                    .get_by_id(plan.next_task_id)
                    .await
                    .context("Failed to load next task for auto-advanced emit")?
                    .ok_or_else(|| {
                        anyhow!(
                            "Next task {} not found after auto-advance",
                            plan.next_task_id
                        )
                    })?;

                let _ = app_handle.emit(
                    domain::event_names::task::AUTO_ADVANCED,
                    json!({
                        "from_task_id": task_id.to_string(),
                        "to_task_id": plan.next_task_id.to_string(),
                        "to_task": to_task,
                    }),
                );
```

Confirm `task_repo` and `anyhow::Context` are already in scope at this site (read the surrounding function signature). Grep: `rg "task_repo" apps/tauri-app/src/commands/task_cmd/complete_flow.rs | head`.

- [ ] **Step 6: Run test to verify it passes**

```
cargo test -p infra auto_advanced_payload_embeds_to_task -- --nocapture
```

Expected: PASS.

- [ ] **Step 7: Verify `tauri-app` still compiles**

```
cargo build -p tauri-app
```

Expected: succeeds with no errors. (This crate is not tested by `cargo test -p infra`; the `complete_flow.rs` change must compile-check separately.)

If the project does not build `tauri-app` on this host (e.g. missing system deps for Tauri), at minimum run `cargo check -p tauri-app`. If that's also unavailable, skip with a note in the commit body — but `complete_flow.rs` is a small, mechanical change; visual inspection of the surrounding `task_repo` scope is sufficient.

- [ ] **Step 8: Run full infra test suite**

```
cargo test -p infra
```

Expected: all pass.

- [ ] **Step 9: Commit**

```bash
git add core/infra/src/adapters/timer/event_handlers/countdown_expired.rs \
        apps/tauri-app/src/commands/task_cmd/complete_flow.rs \
        core/infra/tests/app/task_event_payloads.rs
git commit -m "feat(infra+tauri/task): embed to_task in task:auto_advanced payload"
```

---

### Task 5: Frontend payload types + `useTasks.ts` setters

**Files:**
- Modify: `apps/react-ui/src/pages/tasks/useTasks.ts`
- Modify: `apps/react-ui/src/lib/tauri.ts`

**Interfaces:**
- Produces: `TaskActiveChangedPayload.task`, `TaskCompletedPayload.task`, `TaskResetPayload` (new), `TaskAutoAdvancedPayload.to_task`. `useTaskStore` gains `applyActiveTask(task)` and `applyTaskIfActiveForId(taskId, task)`.

- [ ] **Step 1: Update `useTasks.ts` payload interfaces**

Modify `apps/react-ui/src/pages/tasks/useTasks.ts`:

Add a `task: Task` field to `TaskActiveChangedPayload` and `TaskCompletedPayload`. The final shapes:

```ts
export interface TaskActiveChangedPayload {
  old_task_id: string | null
  new_task_id: string
  workflow_result: string
  version: number
  occurred_at: string
  task: Task
}

export interface TaskCompletedPayload {
  task_id: string
  total_sessions: number
  completed_at: string
  version: number
  occurred_at: string
  task: Task
}

export interface TaskResetPayload {
  task_id: string
  name: string | null
  description: string | null
  max_sessions: number | null
  tags: string[] | null
  version: number
  occurred_at: string
  task: Task
}

export interface TaskAutoAdvancedPayload {
  from_task_id: string
  to_task_id: string
  to_task: Task
}
```

Note: `TaskResetPayload` is new (the current codebase has `'task:task_reset': unknown`). Field names mirror the Rust `Reset` struct (`core/domain/src/task/events/task_reset.rs:6-14`) plus the new `task` field.

- [ ] **Step 2: Add store setters**

In the same file, extend the `TaskStore` interface (around line 79-97) and add two actions:

In the interface:
```ts
interface TaskStore {
  tasks: Task[]
  activeTask: Task | null
  isLoading: boolean
  isBusy: boolean
  error: BackendError | null
  loadTasks: () => Promise<boolean>
  loadActiveTask: () => Promise<boolean>
  applyActiveTask: (task: Task) => void
  applyTaskIfActiveForId: (taskId: string, task: Task) => void
  createTask: (req: CreateTaskRequest) => Promise<boolean>
  // ... unchanged actions ...
}
```

In the store implementation (after `loadActiveTask`):
```ts
  applyActiveTask: (task) => set({ activeTask: task }),

  applyTaskIfActiveForId: (taskId, task) => {
    if (get().activeTask?.id === taskId) {
      set({ activeTask: task })
    }
  },
```

- [ ] **Step 3: Update `tauri.ts` payload map**

Modify `apps/react-ui/src/lib/tauri.ts`:

Update imports (around line 13-20) to include `TaskResetPayload`:
```ts
import type {
  Task,
  CreateTaskRequest,
  UpdateTaskRequest,
  TaskActiveChangedPayload,
  TaskCompletedPayload,
  TaskResetPayload,
  TaskAutoAdvancedPayload,
} from '@/pages/tasks/useTasks'
```

Update the `EventPayloadMap` (around line 168-185):
```ts
  'task:task_reset': TaskResetPayload
```

(Replace the current `'task:task_reset': unknown` line.) The other three task events (`active_changed`, `task_completed`, `auto_advanced`) already reference their payload types; those types now carry the extra `task` / `to_task` field automatically.

- [ ] **Step 4: Verify types + lint**

```
cd apps/react-ui && npm run typecheck && npm run lint
```

Expected: both pass. If `tsc` reports unused-import errors elsewhere (e.g. `TaskResetPayload` was previously inlined as `unknown`), fix by removing the dead code paths. If lint complains about ordering, run `npm run format` first.

- [ ] **Step 5: Commit**

```bash
git add apps/react-ui/src/pages/tasks/useTasks.ts apps/react-ui/src/lib/tauri.ts
git commit -m "feat(react-ui/tasks): add Task-bearing payload types and store setters"
```

---

### Task 6: `useTimer.ts` `applyTimerState` setter

**Files:**
- Modify: `apps/react-ui/src/pages/timer/useTimer.ts`

**Interfaces:**
- Produces: `useTimerStore.applyTimerState(state: TimerStateData)` — sets `timer.state` while preserving `timer.task_id`. No-ops if `timer` is null.

- [ ] **Step 1: Add the action to the store interface**

Modify `apps/react-ui/src/pages/timer/useTimer.ts`:

Add to `TimerStore` interface (around line 127-140):
```ts
interface TimerStore {
  timer: Timer | null
  error: BackendError | null
  isBusy: boolean
  fetchTimer: () => Promise<boolean>
  applyTick: (payload: TickPayload) => void
  applyTimerState: (state: TimerStateData) => void
  start: () => Promise<boolean>
  // ... rest unchanged ...
}
```

- [ ] **Step 2: Implement the action**

In the store (immediately after `applyTick`, around line 165):
```ts
  applyTimerState: (state) => {
    const timer = get().timer
    if (!timer) return
    set({ timer: { task_id: timer.task_id, state } })
  },
```

Note: this preserves the existing `task_id` (which only changes via a `task:active_changed` event). When `timer` is null (initial mount, before the first `fetchTimer` resolves), the action no-ops — the next `fetchTimer` will populate.

- [ ] **Step 3: Verify types + lint**

```
cd apps/react-ui && npm run typecheck && npm run lint
```

Expected: both pass.

- [ ] **Step 4: Commit**

```bash
git add apps/react-ui/src/pages/timer/useTimer.ts
git commit -m "feat(react-ui/timer): add applyTimerState setter for direct event mapping"
```

---

### Task 7: Rewrite `EventBus.ts` to direct-map payloads

**Files:**
- Modify: `apps/react-ui/src/app/EventBus.ts`

**Interfaces:**
- Consumes: `applyActiveTask`, `applyTaskIfActiveForId` (from Task 5); `applyTimerState`, `fetchTimer`, `applyTick` (Task 6 + existing).

- [ ] **Step 1: Rewrite the hook**

Replace the entire contents of `apps/react-ui/src/app/EventBus.ts` with:

```ts
import { useEffect } from 'react'
import { toast } from 'sonner'
import { onEvent, events } from '@/lib/tauri'
import { useTimerStore } from '@/pages/timer/useTimer'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { useTaskStore } from '@/pages/tasks/useTasks'
import { useScreenBlockerStore } from '@/app/useScreenBlocker'

/**
 * Global, always-on backend event subscriptions.
 *
 * Each handler maps its payload directly into the relevant store slice —
 * no IPC round-trip. `fetchTimer` is still called on the four task events
 * because the orchestrators (`switch_active_task`, `reset_task`, etc.) do
 * not emit `timer:*` events after `load_state` (documented gap,
 * docs/superpowers/specs/2026-06-27-task-switch-resets-timer-design.md).
 * Non-calling windows therefore need an explicit timer re-read to stay
 * in sync with the new task's bound timer.
 *
 * Scope rules:
 *  - Timer events are global: the timer must keep reconciling even while
 *    the Timer page is unmounted.
 *  - `task:task_completed` and `task:task_reset` may target a non-active
 *    task; the conditional setter (`applyTaskIfActiveForId`) leaves
 *    `activeTask` untouched in that case.
 *  - `timer:status_changed` is covered by the start/pause/resume/reset
 *    family below (no separate handler needed here).
 */
export function useEventBus(): void {
  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const applyTick = useTimerStore((s) => s.applyTick)
  const applyTimerState = useTimerStore((s) => s.applyTimerState)
  const applyActiveTask = useTaskStore((s) => s.applyActiveTask)
  const applyTaskIfActiveForId =
    useTaskStore((s) => s.applyTaskIfActiveForId)

  useEffect(() => {
    fetchTimer()

    const unlisteners: Array<Promise<UnlistenFn>> = [
      // Real-time countdown; pure local state update.
      onEvent(events.timerTick, applyTick),

      // Timer lifecycle: payload is `TimerStateData`; preserve task_id.
      onEvent(events.timerPhaseCompleted, applyTimerState),
      onEvent(events.timerReset, applyTimerState),
      onEvent(events.timerPaused, applyTimerState),
      onEvent(events.timerStarted, applyTimerState),
      onEvent(events.timerResumed, applyTimerState),

      // Task events: direct-map the embedded Task; re-fetch the timer
      // because the timer state is bound to the active task and the
      // orchestrator does not emit a timer:* event after load_state.
      onEvent(events.taskActiveChanged, (payload) => {
        if (payload) {
          applyActiveTask(payload.task)
          fetchTimer()
        }
      }),
      onEvent(events.taskCompleted, (payload) => {
        applyTaskIfActiveForId(payload.task_id, payload.task)
        fetchTimer()
        toast.success('Task completed!')
      }),
      onEvent(events.taskReset, (payload) => {
        applyTaskIfActiveForId(payload.task_id, payload.task)
        fetchTimer()
        toast.info('Task progress reset')
      }),
      onEvent(events.taskAutoAdvanced, (payload) => {
        applyActiveTask(payload.to_task)
        fetchTimer()
        toast.success('Switched to next task')
      }),

      // Screen blocker: show the focus-enforcement overlay.
      onEvent(events.screenBlockerActivate, (payload) => {
        useScreenBlockerStore.getState().activate(payload.message)
      }),
    ]

    return () => {
      for (const p of unlisteners) void p.then((fn) => fn())
    }
  }, [
    fetchTimer,
    applyTick,
    applyTimerState,
    applyActiveTask,
    applyTaskIfActiveForId,
  ])
}
```

Key changes vs. the prior version:
- Removes `createBatchedLoader` import + usage (no batched fetches to coalesce).
- Removes the `setTimeout(..., 500)` hack (no longer waiting on backend commit; the active task is derived from the payload).
- Removes `loadActiveTask` from the dependency array (no longer used here).
- `task:active_changed` payload is now non-null typed (`TaskActiveChangedPayload`); the `if (payload)` guard is defensive.

- [ ] **Step 2: Verify types + lint**

```
cd apps/react-ui && npm run typecheck && npm run lint
```

Expected: both pass. If `tsc` flags an unused import (`createBatchedLoader` removal, `useTaskStore.loadActiveTask` selector removal), the rewrite above already drops them.

- [ ] **Step 3: Manual smoke test**

Build + launch the app and exercise each path. From the workspace root:

```
cargo tauri dev
```

(If the dev command name differs, check `apps/tauri-app/tauri.conf.json` or `apps/tauri-app/package.json` for the dev script.)

Smoke-test checklist:
1. **Start a session** on a task → countdown begins (tick handler works).
2. **Switch active task from the tray** (non-calling window path) → Timer page shows the new task name and a fresh `Idle` timer (active_changed + fetchTimer).
3. **Complete a task** via the UI → toast shows; `activeTask` updates to the completed task; if auto-advance fires, `activeTask` becomes the next task (auto_advanced + fetchTimer).
4. **Reset a task** → toast shows; if it was the active task, session dots reset (task_reset + fetchTimer).
5. **Pause / resume / reset the timer** via UI → timer state changes immediately (applyTimerState).
6. **Pause then Resume** from the tray → timer page reflects the change without a 500 ms lag.

If the build doesn't run in this environment, document the smoke test as a follow-up in the commit body.

- [ ] **Step 4: Commit**

```bash
git add apps/react-ui/src/app/EventBus.ts
git commit -m "refactor(react-ui/app): direct-map event payloads to store state in EventBus"
```

---

## Self-Review Notes

- **Spec coverage:** All 6 timer events and 4 task events from the spec's field-mapping table are handled in Task 7. Backend patches in Tasks 1-4 produce the embedded `Task` / `to_task` fields the frontend consumes in Tasks 5-7.
- **Frontend unit tests:** Deliberately omitted. The React UI has no test framework (`apps/react-ui/package.json` has no `test` script and no vitest/jest dependency). Adding one is out of scope. The store setters (`applyActiveTask`, `applyTaskIfActiveForId`, `applyTimerState`) are 1-3 lines each; TypeScript types + manual smoke are the verification surface.
- **`fetchTimer` retained on task events:** per spec § "Chosen Design — Timer reconciliation". Documented in the new `EventBus.ts` header comment.
- **500 ms `setTimeout` removed:** per spec. The active-task slice no longer depends on a backend commit race; the timer slice is still IPC-fetched but the `fetchTimer` coalescing in `useTimer.ts:147-153` handles bursts.
- **Risks:** If a `task:auto_advanced` event arrives before the corresponding `task:active_changed`, `applyActiveTask(payload.to_task)` is still correct (both reference the same new active task). The order-independence is a property of the new design.
