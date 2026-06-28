# Phase-completed task + auto-advanced timer + useTasks consolidation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Embed `task` in every `timer:phase_completed` payload and `timer` in every `task:auto_advanced` payload; consolidate all task-event subscriptions into the global `EventBus.ts` and delete `useTasksEventBus`.

**Architecture:** Backend emitters gain one extra field each (additive for AUTO_ADVANCED, envelope for PHASE_COMPLETED). Frontend `EventBus.ts` unwraps the new shapes and gains two new subscriptions (`taskProgressUpdated`, `taskListUpdated`) pulled in from the soon-to-be-deleted `useTasksEventBus`. The `fetchTimer()` round-trip is removed from the AUTO_ADVANCED handler only.

**Tech Stack:** Rust (infra, tauri-app crates); React + TypeScript + Zustand (react-ui).

## Global Constraints

- **Tick-loop ownership contract** (`CLAUDE.md`): handlers are UI-only emitters; never mutate `TimerTickService::cancel_handle`. No `tokio::time::sleep` to "drain" handlers.
- **Conventional Commits with scope**, e.g. `feat(infra/timer):`, `refactor(react-ui/app):`.
- **Backend test command:** `cargo test -p infra` (run from repo root).
- **Frontend verify commands** (run from `apps/react-ui`): `npm run typecheck`, `npm run lint`, `npm run build`. No JS test framework exists.
- **Backend payload tests** live in `core/infra/tests/app/task_event_payloads.rs`. Existing helpers: `setup_ctx(name).await`, `AppContextBuilder::new().with_name(name).build().await`, `TaskBuilder::new()`, `ctx.ui_simulator.app_handle().events_of_type(name)`, `ctx.ui_simulator.app_handle().clear_events()`.
- **`Timer::state()`** returns `&TimerState` which serializes to the frontend `TimerStateData` shape (`{state, remaining_seconds?, paused_from?}`). The full `Timer` serializes as `{task_id, state}`. The existing emit code `self.timer_srv.with_timer(|t| json!(t.state()))` therefore yields a `TimerStateData`-shaped JSON value — exactly what the frontend `applyTimerState` expects.
- **`PhaseSkipped` event** carries `task_id: TaskId` directly (see `core/domain/src/timer/events/phase_skipped.rs:8`). Use this, not the timer state.
- **`register_timer_handlers`** already takes `task_repo` as a parameter (see `core/infra/src/adapters/timer/event_handlers/registry.rs:21`). The test harness already passes it through (see `core/infra/tests/core/mocks/ui/register_event_handlers.rs:37`). Adding `task_repo` to `PhaseSkippedHandler::new()` requires only one wiring change at `registry.rs:32-35`.
- **Scope:** `applyTimerState`, `applyActiveTask`, `applyTaskIfActiveForId` setter signatures are unchanged. `fetchTimer()` stays on `taskActiveChanged`, `taskCompleted`, `taskReset` (out of scope to remove).

---

## File Structure

### Backend (Rust)
- **Modify** `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs` — Tasks 1, 3 (PHASE_COMPLETED Started envelope, AUTO_ADVANCED timer in both arms)
- **Modify** `core/infra/src/adapters/timer/event_handlers/phase_skipped.rs` — Task 2 (add `task_repository`, embed task)
- **Modify** `core/infra/src/adapters/timer/event_handlers/registry.rs` — Task 2 (pass `task_repo` to `PhaseSkippedHandler::new()`)
- **Modify** `apps/tauri-app/src/commands/task_cmd/complete_flow.rs` — Task 3 (embed timer in AUTO_ADVANCED)
- **Modify** `core/infra/tests/app/task_event_payloads.rs` — Tasks 1, 2, 3 (new tests)

### Frontend (TypeScript)
- **Modify** `apps/react-ui/src/pages/tasks/useTasks.ts` — Task 4 (new `PhaseCompletedPayload`, extend `TaskAutoAdvancedPayload`); Task 6 (delete `useTasksEventBus`)
- **Modify** `apps/react-ui/src/lib/tauri.ts` — Task 4 (update `EventPayloadMap['timer:phase_completed']`)
- **Modify** `apps/react-ui/src/app/EventBus.ts` — Task 5 (handlers + new subscriptions)
- **Modify** `apps/react-ui/src/pages/tasks/TasksPage.tsx` — Task 6 (remove `useTasksEventBus()` call)

---

## Task 1: PHASE_COMPLETED envelope — countdown_expired.rs Started arm

**Files:**
- Modify: `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs:108-117`
- Test: `core/infra/tests/app/task_event_payloads.rs` (append)

**Interfaces:**
- Consumes: existing `task: Task` destructured from `PhaseOutcome::Started` at L86-92; existing `state_json` computed at L108-109.
- Produces: `timer:phase_completed` payload shape becomes `{ timer: <TimerState>, task: <Task> }` (envelope). Downstream consumers (frontend `EventBus.ts`, updated in Task 5) must unwrap.

- [ ] **Step 1: Add failing test**

Append to `core/infra/tests/app/task_event_payloads.rs`:

```rust
/// `timer:phase_completed` payload must carry both the new timer state
/// AND the updated bound Task so the React EventBus can update both
/// slices without an IPC round-trip. Verified via the natural-expiry
/// path (CountdownExpiredHandler Started arm).
#[tokio::test]
async fn phase_completed_payload_embeds_task_and_timer() {
    use domain::{timer::events::CountdownExpired, Phase};
    use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

    let ctx = AppContextBuilder::new()
        .with_name("phase_completed_payload_embeds_task_and_timer")
        .build()
        .await
        .expect("build ctx");

    let task = TaskBuilder::new()
        .name("Embed me")
        .max_sessions(3)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    let task_id = task.id();
    let expected_name = task.name().to_string();
    ctx.task_repo.create(task).await.unwrap();

    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd { task_id: Some(task_id) },
    )
    .await
    .expect("start");

    tokio::time::sleep(Duration::from_millis(100)).await;
    ctx.ui_simulator.app_handle().clear_events();

    // Natural work-phase expiry drives CountdownExpiredHandler Started arm.
    ctx.event_bus
        .publish(Box::new(CountdownExpired::new(Phase::Work, task_id)));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::timer::PHASE_COMPLETED);
    assert!(
        !events.is_empty(),
        "timer:phase_completed was not emitted"
    );

    let payload = &events[0].payload;
    let timer = payload
        .get("timer")
        .expect("payload missing `timer` envelope field");
    assert!(
        timer.get("state").is_some(),
        "timer field must carry TimerState"
    );
    let task = payload
        .get("task")
        .expect("payload missing `task` envelope field");
    assert_eq!(task["id"], task_id.to_string());
    assert_eq!(task["name"], expected_name);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p infra phase_completed_payload_embeds_task_and_timer`
Expected: FAIL — `payload missing 'timer' envelope field` (current payload is bare `TimerState`).

- [ ] **Step 3: Apply the envelope change**

In `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs`, replace the PHASE_COMPLETED emit block in the `PhaseOutcome::Started` arm (currently L111-117):

```rust
                self.emitter
                    .emit(
                        ui_listeners::timer::PHASE_COMPLETED,
                        json!({ "timer": state_json, "task": task }),
                    )
                    .map_err(|e| domain::Error::EventPublishingError {
                        message: format!(
                            "Failed to emit timer phase completed event: {e}"
                        ),
                    })?;
```

(Note the original `state_json` variable from L108-109 is reused; `task` is the destructured `PhaseOutcome::Started { task, .. }`.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p infra phase_completed_payload_embeds_task_and_timer`
Expected: PASS.

- [ ] **Step 5: Run full infra suite to catch regressions**

Run: `cargo test -p infra`
Expected: All tests pass (104 + 1 doctest).

- [ ] **Step 6: Commit**

```bash
git add core/infra/src/adapters/timer/event_handlers/countdown_expired.rs core/infra/tests/app/task_event_payloads.rs
git commit -m "feat(infra/timer): embed task in phase_completed payload (countdown_expired)"
```

---

## Task 2: PHASE_COMPLETED envelope — phase_skipped.rs + registry wiring

**Files:**
- Modify: `core/infra/src/adapters/timer/event_handlers/phase_skipped.rs` (struct + constructor + handle body)
- Modify: `core/infra/src/adapters/timer/event_handlers/registry.rs:32-35` (pass `task_repo.clone()`)
- Test: `core/infra/tests/app/task_event_payloads.rs` (append)

**Interfaces:**
- Consumes: `task_repo: Arc<dyn TaskRepository + Send + Sync>` (already in `register_timer_handlers` scope at `registry.rs:21`); `phase_skipped.task_id` from the event payload (see `core/domain/src/timer/events/phase_skipped.rs:8`).
- Produces: same `{ timer, task }` envelope as Task 1. Both `timer:phase_skipped` and `timer:phase_completed` from this handler carry the envelope (callers expecting the prior bare `TimerState` must be updated — none exist in this codebase except the frontend, updated in Task 5).

- [ ] **Step 1: Add failing test**

Append to `core/infra/tests/app/task_event_payloads.rs`:

```rust
/// Manual phase skip must also embed the bound Task in PHASE_COMPLETED so
/// the payload shape is uniform across emitters. The task is unchanged
/// (skipping a phase does not increment sessions), but the field must be
/// present and consistent with the natural-expiry path.
#[tokio::test]
async fn phase_skipped_payload_embeds_task() {
    use domain::{timer::events::PhaseSkipped, Phase};

    let ctx = setup_ctx("phase_skipped_payload_embeds_task").await;

    let task = TaskBuilder::new()
        .name("Skip me")
        .max_sessions(3)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    let task_id = task.id();
    let expected_name = task.name().to_string();
    ctx.task_repo.create(task).await.unwrap();

    // Bind the task to the timer so the handler can read its state.
    use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};
    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd { task_id: Some(task_id) },
    )
    .await
    .expect("start");

    tokio::time::sleep(Duration::from_millis(100)).await;
    ctx.ui_simulator.app_handle().clear_events();

    ctx.event_bus.publish(Box::new(PhaseSkipped::new(
        task_id,
        Phase::Work,
        Phase::ShortBreak,
        1,
    )));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::timer::PHASE_COMPLETED);
    assert!(
        !events.is_empty(),
        "timer:phase_completed was not emitted from PhaseSkippedHandler"
    );

    let payload = &events[0].payload;
    assert!(
        payload.get("timer").is_some(),
        "payload missing `timer` envelope field"
    );
    let embedded = payload
        .get("task")
        .expect("payload missing `task` envelope field");
    assert_eq!(embedded["id"], task_id.to_string());
    assert_eq!(embedded["name"], expected_name);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p infra phase_skipped_payload_embeds_task`
Expected: FAIL — `payload missing 'timer' envelope field` (current emit sends bare `TimerState`).

- [ ] **Step 3: Add `task_repository` field to `PhaseSkippedHandler`**

In `core/infra/src/adapters/timer/event_handlers/phase_skipped.rs`, replace the entire file content with:

```rust
use async_trait::async_trait;
use domain::{Event, PhaseSkipped, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::TimerTickService;
use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

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
        Self {
            emitter,
            timer_srv,
            task_repository,
        }
    }
}

#[async_trait]
impl EventHandler for PhaseSkippedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<PhaseSkipped>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let phase_skipped = event
            .as_any()
            .downcast_ref::<domain::PhaseSkipped>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to skip phase".to_string(),
            })?;

        self.timer_srv.load_state().await?;
        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

        // Embed the bound Task so the payload shape matches the
        // natural-expiry path in CountdownExpiredHandler. The task itself
        // is unchanged by a skip; this is purely for shape consistency.
        let task_json = match self
            .task_repository
            .get_by_id(phase_skipped.task_id)
            .await
        {
            Ok(Some(task)) => json!(task),
            Ok(None) => {
                log::warn!(
                    "PhaseSkippedHandler: task {} not found; emitting task: null",
                    phase_skipped.task_id
                );
                json!(null)
            }
            Err(e) => {
                log::warn!(
                    "PhaseSkippedHandler: failed to load task {}: {e}; emitting task: null",
                    phase_skipped.task_id
                );
                json!(null)
            }
        };

        let payload = json!({ "timer": state_json, "task": task_json });

        self.emitter
            .emit(
                domain::event_names::timer::PHASE_SKIPPED,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit phase skipped event: {e}"),
            })?;

        self.emitter
            .emit(domain::event_names::timer::PHASE_COMPLETED, payload)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit phase skipped event: {e}"),
            })?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "PhaseSkippedHandler"
    }
}

impl From<PhaseSkippedHandler> for Box<dyn EventHandler> {
    fn from(handler: PhaseSkippedHandler) -> Self {
        Box::new(handler)
    }
}
```

- [ ] **Step 4: Update registry wiring**

In `core/infra/src/adapters/timer/event_handlers/registry.rs`, update the `PhaseSkippedHandler::new(...)` call (currently L32-35) to pass `task_repo.clone()`:

```rust
    event_bus.subscribe(Box::new(PhaseSkippedHandler::new(
        emitter.clone(),
        timer_srv.clone(),
        task_repo.clone(),
    )))?;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p infra phase_skipped_payload_embeds_task`
Expected: PASS.

- [ ] **Step 6: Run full infra suite**

Run: `cargo test -p infra`
Expected: All pass (105 + 1 doctest).

- [ ] **Step 7: Commit**

```bash
git add core/infra/src/adapters/timer/event_handlers/phase_skipped.rs core/infra/src/adapters/timer/event_handlers/registry.rs core/infra/tests/app/task_event_payloads.rs
git commit -m "feat(infra/timer): embed task in phase_completed payload (phase_skipped)"
```

---

## Task 3: AUTO_ADVANCED timer field (3 emit sites)

**Files:**
- Modify: `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs:119-134` (Started arm), `:178-193` (Paused arm)
- Modify: `apps/tauri-app/src/commands/task_cmd/complete_flow.rs:156-163`
- Test: `core/infra/tests/app/task_event_payloads.rs` (append)

**Interfaces:**
- Consumes: `state_json` (Task 1's variable in Started arm, still in scope at L119-134); `timer.state()` for the Paused arm (destructured at L160 from `PhaseOutcome::Paused`); `timer_tick_service.with_timer(...)` for `complete_flow.rs`.
- Produces: `task:auto_advanced` payload gains a `timer: <TimerState>` field alongside the existing `from_task_id`, `to_task_id`, `to_task`.

- [ ] **Step 1: Add failing test**

Append to `core/infra/tests/app/task_event_payloads.rs`:

```rust
/// `task:auto_advanced` payload must carry the new task's timer state so
/// the React EventBus can applyTimerState directly without `fetchTimer`.
#[tokio::test]
async fn auto_advanced_payload_embeds_timer() {
    use domain::{TaskCyclingBehavior, timer::events::CountdownExpired};
    use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

    let ctx = AppContextBuilder::new()
        .with_name("auto_advanced_payload_embeds_timer")
        .build()
        .await
        .expect("build ctx");

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

    // Work expiry consumes task1's only session; transitions to ShortBreak.
    ctx.event_bus
        .publish(Box::new(CountdownExpired::new(Phase::Work, task1_id)));
    tokio::time::sleep(Duration::from_millis(300)).await;

    // ShortBreak expiry finalizes task1 and cycles to task2 → emits
    // task:auto_advanced with the new task's timer state embedded.
    ctx.event_bus
        .publish(Box::new(CountdownExpired::new(Phase::ShortBreak, task1_id)));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::AUTO_ADVANCED);
    assert!(!events.is_empty(), "task:auto_advanced was not emitted");

    let payload = &events[0].payload;
    let timer = payload
        .get("timer")
        .expect("payload missing `timer` field");
    assert!(
        timer.get("state").is_some(),
        "timer field must carry TimerState"
    );
    // The post-cycle timer should be bound to task2.
    assert_eq!(timer["task_id"], task2_id.to_string());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p infra auto_advanced_payload_embeds_timer`
Expected: FAIL — `payload missing 'timer' field`.

- [ ] **Step 3: Update AUTO_ADVANCED emit in countdown_expired.rs Started arm**

In `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs`, replace the AUTO_ADVANCED emit in the `PhaseOutcome::Started` arm (currently L119-134).

> **Note:** The `timer` field embeds the **full `Timer`** (not just `TimerState`) so the frontend can reconcile `task_id` when a cycle swaps the bound task. `Timer`'s custom `Serialize` impl (`core/domain/src/timer/timer.rs:70-93`) yields `{task_id, state}`. This is intentional — `PhaseCompletedPayload.timer` is `TimerStateData` (task unchanged), but `TaskAutoAdvancedPayload.timer` is the full `Timer` (task changed).

```rust
                if let Some(to_task_id) = cycled_to {
                    self.emitter
                        .emit(
                            ui_listeners::task::AUTO_ADVANCED,
                            json!({
                                "from_task_id": countdown_expired.task_id,
                                "to_task_id": to_task_id,
                                "to_task": task,
                                "timer": self.timer_srv.with_timer(|t| json!(t)).await,
                            }),
                        )
                        .map_err(|e| domain::Error::EventPublishingError {
                            message: format!(
                                "Failed to emit auto-advanced event: {e}"
                            ),
                        })?;
                }
```

`json!(t)` where `t: &Timer` invokes `Timer`'s custom `Serialize`. The `.await` resolves inside the `json!` because `json!` evaluates arguments eagerly. `state_json` (from L108-109) is still used by the PHASE_COMPLETED emit — do not remove it.

- [ ] **Step 4: Update AUTO_ADVANCED emit in countdown_expired.rs Paused arm**

Replace the AUTO_ADVANCED emit in the `PhaseOutcome::Paused` arm (currently L178-193). The `timer` variable is destructured at L160 from `PhaseOutcome::Paused { task, timer, .. }` and is a full `Timer`:

```rust
                if let Some(to_task_id) = cycled_to {
                    self.emitter
                        .emit(
                            ui_listeners::task::AUTO_ADVANCED,
                            json!({
                                "from_task_id": countdown_expired.task_id,
                                "to_task_id": to_task_id,
                                "to_task": task,
                                "timer": timer,
                            }),
                        )
                        .map_err(|e| domain::Error::EventPublishingError {
                            message: format!(
                                "Failed to emit auto-advanced event: {e}"
                            ),
                        })?;
                }
```

If the compiler reports a move error (because the prior STATUS_CHANGED emit at L170 uses `timer.state()` — a borrow, but verify), use `json!(&timer)` instead.

- [ ] **Step 5: Update AUTO_ADVANCED emit in complete_flow.rs**

In `apps/tauri-app/src/commands/task_cmd/complete_flow.rs`, replace the emit block (currently L156-163). Serialize the full `Timer`:

```rust
                let timer_json = timer_tick_service
                    .with_timer(|t| json!(t))
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

`timer_tick_service.load_state()` was called at L90-93 before this block, so the in-memory cache reflects the new task's timer.

- [ ] **Step 6: Verify tauri-app still compiles**

Run: `cargo check -p tauri-app`
Expected: no errors.

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test -p infra auto_advanced_payload_embeds_timer`
Expected: PASS.

- [ ] **Step 8: Run full infra suite**

Run: `cargo test -p infra`
Expected: All pass (106 + 1 doctest).

- [ ] **Step 9: Commit**

```bash
git add core/infra/src/adapters/timer/event_handlers/countdown_expired.rs apps/tauri-app/src/commands/task_cmd/complete_flow.rs core/infra/tests/app/task_event_payloads.rs
git commit -m "feat(infra+tauri/task): embed timer in auto_advanced payload"
```

---

## Task 4: Frontend payload types

**Files:**
- Modify: `apps/react-ui/src/pages/tasks/useTasks.ts:56-89` (payload type block)
- Modify: `apps/react-ui/src/lib/tauri.ts:178` (EventPayloadMap entry)

**Interfaces:**
- Consumes: existing `Task` interface (`useTasks.ts:19-31`); existing `TimerStateData` interface (`useTimer.ts:22-26`).
- Produces: `PhaseCompletedPayload` (new export from `useTasks.ts`); `TaskAutoAdvancedPayload.timer` field (additive); updated `EventPayloadMap['timer:phase_completed']` typing.

- [ ] **Step 1: Add `PhaseCompletedPayload` and extend `TaskAutoAdvancedPayload`**

In `apps/react-ui/src/pages/tasks/useTasks.ts`:

Add a new import for `TimerStateData` AND `Timer` after the existing imports (after L8):

```ts
import type { TimerStateData, Timer } from '@/pages/timer/useTimer'
```

Add the `PhaseCompletedPayload` interface and extend `TaskAutoAdvancedPayload`. Replace the existing `TaskAutoAdvancedPayload` block (currently L85-89):

```ts
export interface PhaseCompletedPayload {
  timer: TimerStateData
  task: Task
}

export interface TaskAutoAdvancedPayload {
  from_task_id: string
  to_task_id: string
  to_task: Task
  timer: Timer
}
```

> **Why the two payloads have different `timer` shapes:** `PhaseCompletedPayload.timer` is `TimerStateData` (just the state machine — task is unchanged when a phase completes, so the existing `applyTimerState` setter which preserves `task_id` is correct). `TaskAutoAdvancedPayload.timer` is the full `Timer` (with `task_id` + `state`) because a cycle swaps the bound task, and the frontend must reconcile the new `task_id`. This matches the backend emit shapes: Tasks 1+2 emit `json!(t.state())` for PHASE_COMPLETED's inner timer; Task 3 emits `json!(t)` (full Timer) for AUTO_ADVANCED's timer field.

- [ ] **Step 2: Update EventPayloadMap entry**

In `apps/react-ui/src/lib/tauri.ts`, add `PhaseCompletedPayload` to the existing task-related type import block (currently L13-21):

Before:
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

After (insert `PhaseCompletedPayload` alphabetically):
```ts
import type {
  Task,
  CreateTaskRequest,
  UpdateTaskRequest,
  PhaseCompletedPayload,
  TaskActiveChangedPayload,
  TaskCompletedPayload,
  TaskResetPayload,
  TaskAutoAdvancedPayload,
} from '@/pages/tasks/useTasks'
```

Do NOT remove the `TimerStateData` import at L8 — it is still used by other entries (`timer:timer_reset`, `timer:timer_started`, `timer:timer_paused`, `timer:timer_resumed`).

Change L178 from:

```ts
  'timer:phase_completed': TimerStateData,
```

to:

```ts
  'timer:phase_completed': PhaseCompletedPayload,
```

- [ ] **Step 3: Verify types compile**

Run (from `apps/react-ui`): `npm run typecheck`
Expected: PASS (no type errors).

Note: `EventBus.ts` will have type errors after this step because it currently passes `applyTimerState` directly as the PHASE_COMPLETED handler — `applyTimerState` expects `TimerStateData`, but the payload is now `PhaseCompletedPayload`. Those errors will be resolved in Task 5. **If typecheck fails with only that one error, proceed.** If it fails with anything else, investigate.

- [ ] **Step 4: Commit**

```bash
git add apps/react-ui/src/pages/tasks/useTasks.ts apps/react-ui/src/lib/tauri.ts
git commit -m "feat(react-ui/tasks): add PhaseCompletedPayload and timer field on auto_advanced"
```

---

## Task 5: EventBus handler updates

**Files:**
- Modify: `apps/react-ui/src/pages/timer/useTimer.ts` (add `applyTimer` setter to interface + impl)
- Modify: `apps/react-ui/src/app/EventBus.ts` (full body rewrite of the `unlisteners` array + doc-comment)

**Interfaces:**
- Consumes: `PhaseCompletedPayload` (Task 4); extended `TaskAutoAdvancedPayload` with full-`Timer` `timer` field (Task 4); existing setters `applyTimerState`, `applyActiveTask`, `applyTaskIfActiveForId`; existing `fetchTimer`; existing `useTaskStore.getState().loadTasks` + `loadActiveTask`; existing `useScreenBlockerStore.getState().activate`.
- Produces: a new `applyTimer(timer: Timer)` setter on `useTimerStore` (replaces the whole timer including `task_id`); a `useEventBus` hook that maps every task/timer event without IPC (except `taskListUpdated`'s full reload).

- [ ] **Step 1: Add `applyTimer` setter to useTimerStore**

In `apps/react-ui/src/pages/timer/useTimer.ts`:

Find the `TimerStore` interface (around L100-140, the `applyTimerState` line). After the `applyTimerState` declaration, add a new `applyTimer` declaration:

```ts
  applyTimerState: (state: TimerStateData) => void
  applyTimer: (timer: Timer) => void
```

Find the implementation (around L170-180, the existing `applyTimerState: (state) => {` block). Immediately after it, add the `applyTimer` impl:

```ts
  applyTimer: (timer) => set({ timer }),
```

The `applyTimerState` setter preserves `task_id` (unchanged); `applyTimer` replaces the whole `timer` field (including `task_id`). The two are intentionally distinct: `applyTimerState` is for events where the task binding hasn't changed (phase completion, reset, pause, etc.); `applyTimer` is for events where the task binding HAS changed (auto-advance).

- [ ] **Step 2: Rewrite EventBus.ts**

Replace the entire content of `apps/react-ui/src/app/EventBus.ts` with:

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
 * no IPC round-trip except `task:list_updated` (whose backend payload
 * shape is heterogeneous across 7 emitters and cannot be direct-mapped).
 *
 * `fetchTimer` is still called on three task events (`active_changed`,
 * `task_completed`, `task_reset`) because their payloads do not carry
 * timer state and the orchestrators do not emit `timer:*` after
 * `load_state` (documented gap,
 * docs/superpowers/specs/2026-06-27-task-switch-resets-timer-design.md).
 *
 * Scope rules:
 *  - Timer events are global: the timer must keep reconciling even while
 *   the Timer page is unmounted.
 *  - `task:task_completed` and `task:task_reset` may target a non-active
 *   task; the conditional setter (`applyTaskIfActiveForId`) leaves
 *   `activeTask` untouched in that case.
 *  - `applyTimerState` (preserves task_id) is used for events where the
 *   bound task is unchanged; `applyTimer` (replaces whole timer) is used
 *   for `task:auto_advanced` where the cycle swaps the bound task.
 */
export function useEventBus(): void {
  const fetchTimer = useTimerStore((s) => s.fetchTimer)
  const applyTick = useTimerStore((s) => s.applyTick)
  const applyTimerState = useTimerStore((s) => s.applyTimerState)
  const applyTimer = useTimerStore((s) => s.applyTimer)
  const applyActiveTask = useTaskStore((s) => s.applyActiveTask)
  const applyTaskIfActiveForId = useTaskStore((s) => s.applyTaskIfActiveForId)

  useEffect(() => {
    fetchTimer()

    const unlisteners: Array<Promise<UnlistenFn>> = [
      // Real-time countdown; pure local state update.
      onEvent(events.timerTick, applyTick),

      // PHASE_COMPLETED now carries { timer, task } (envelope). The inner
      // `timer` is a bare TimerStateData — task is unchanged, so we
      // preserve the existing task_id via applyTimerState.
      onEvent(events.timerPhaseCompleted, (payload) => {
        applyTimerState(payload.timer)
        applyTaskIfActiveForId(payload.task.id, payload.task)
      }),

      // Timer lifecycle: payload is bare TimerStateData; preserve task_id.
      onEvent(events.timerReset, applyTimerState),
      onEvent(events.timerPaused, applyTimerState),
      onEvent(events.timerStarted, applyTimerState),
      onEvent(events.timerResumed, applyTimerState),

      // Task events: direct-map the embedded Task; re-fetch the timer
      // for the three events whose payload does not carry timer state.
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

      // AUTO_ADVANCED carries the new task AND the full new Timer (with
      // new task_id) — applyTimer replaces both fields. No IPC.
      onEvent(events.taskAutoAdvanced, (payload) => {
        applyActiveTask(payload.to_task)
        applyTimer(payload.timer)
        toast.success('Switched to next task')
      }),

      // PROGRESS_UPDATED carries a bare Task — the only signal for
      // current_sessions increments when no cycle occurs.
      onEvent(events.taskProgressUpdated, (task) => {
        applyTaskIfActiveForId(task.id, task)
      }),

      // LIST_UPDATED payload is heterogeneous across 7 emitters — treat
      // as opaque dirty signal and reload both slices. `loadActiveTask`
      // is needed because some mutations (e.g. edit active task's name)
      // emit only this event.
      onEvent(events.taskListUpdated, () => {
        const s = useTaskStore.getState()
        void s.loadTasks()
        void s.loadActiveTask()
      }),

      // Screen blocker: show the focus-enforcement overlay.
      onEvent(events.screenBlockerActivate, (payload) => {
        useScreenBlockerStore.getState().activate(payload.message)
      }),
    ]

    return () => {
      for (const p of unlisteners) void p.then((fn) => fn())
    }
  }, [fetchTimer, applyTick, applyTimerState, applyTimer, applyActiveTask, applyTaskIfActiveForId])
}
```

- [ ] **Step 3: Verify typecheck passes**

Run (from `apps/react-ui`): `npm run typecheck`
Expected: PASS (resolves the temporary error from Task 4 Step 3).

- [ ] **Step 4: Verify lint + build**

Run (from `apps/react-ui`): `npm run lint && npm run build`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/react-ui/src/pages/timer/useTimer.ts apps/react-ui/src/app/EventBus.ts
git commit -m "refactor(react-ui/app): direct-map new payloads + add taskProgressUpdated/taskListUpdated subscriptions"
```

---

## Task 6: Delete useTasksEventBus + TasksPage cleanup

**Files:**
- Modify: `apps/react-ui/src/pages/tasks/useTasks.ts` (delete imports + `useTasksEventBus` function)
- Modify: `apps/react-ui/src/pages/tasks/TasksPage.tsx:25` (remove call)

**Interfaces:**
- Consumes: nothing new. The `task:list_updated` and `task:progress_updated` subscriptions are now in `EventBus.ts` (Task 5).
- Produces: a single global event bus. `TasksPage` reads from stores only.

- [ ] **Step 1: Delete `useTasksEventBus` from `useTasks.ts`**

In `apps/react-ui/src/pages/tasks/useTasks.ts`, make these four edits:

**Edit 1.1 — Drop `onEvent, events` from the tauri import** (currently L4). Change:
```ts
import { invokeCmd, onEvent, events } from '@/lib/tauri'
```
to:
```ts
import { invokeCmd } from '@/lib/tauri'
```
(The store actions only use `invokeCmd` for IPC; `onEvent`/`events` are unused after this task.)

**Edit 1.2 — Delete the `createBatchedLoader` import** (currently L7):
```ts
import { createBatchedLoader } from '@/lib/async'
```

**Edit 1.3 — Delete the `UnlistenFn` import** (currently L9):
```ts
import type { UnlistenFn } from '@tauri-apps/api/event'
```

**Edit 1.4 — Delete the entire `useTasksEventBus` function** (the `export function useTasksEventBus(): void {` block, plus its preceding blank line, through the closing `}`).

- [ ] **Step 2: Remove `useTasksEventBus()` call from TasksPage**

In `apps/react-ui/src/pages/tasks/TasksPage.tsx`, remove the call (currently L25) and its import. Search the file for `useTasksEventBus` and remove all references.

- [ ] **Step 3: Verify typecheck + lint + build**

Run (from `apps/react-ui`):
```bash
npm run typecheck && npm run lint && npm run build
```
Expected: PASS. If typecheck fails with "X is declared but its value is never read" on an import, remove that import too.

- [ ] **Step 4: Commit**

```bash
git add apps/react-ui/src/pages/tasks/useTasks.ts apps/react-ui/src/pages/tasks/TasksPage.tsx
git commit -m "refactor(react-ui/tasks): delete useTasksEventBus; consolidate subscriptions in EventBus"
```

---

## Verification (whole-branch)

After all 6 tasks land, verify before merging:

- [ ] `cargo test -p infra` — full backend suite passes.
- [ ] `cargo check -p tauri-app` — no compile errors.
- [ ] `npm run typecheck && npm run lint && npm run build` (from `apps/react-ui`) — all clean.

## Manual smoke test (deferred to human — no Tauri runtime in subagent env)

Run in `cargo tauri dev`:

1. **Natural work-phase completion:** Start a work session, let it expire. Tasks page shows incremented `current_sessions` without manual reload; Timer page shows next phase.
2. **Manual phase skip:** Click skip button on Timer page. No console errors. Tasks page still shows the same active task (no session increment).
3. **Auto-advance with `AutoAdvance` configured:** Exhaust a task naturally (configure `max_sessions: 1`). On cycle, Timer page updates to the new task + its timer state with no visible IPC latency (verify in DevTools network panel: no `get_timer` call after `task:auto_advanced`).
4. **Manual complete from tray (with `AutoAdvance`):** Same observation as #3 via the tray menu path.
5. **Edit active task's name on Tasks page:** Timer page header updates without a navigation/reload (proves `loadActiveTask` on `task:list_updated` is preserved).
6. **Create / edit / delete a task on Tasks page:** List reloads correctly.

## Risks revisited

- **`TimerStateData` import path:** verify `@/pages/timer/useTimer` exports `TimerStateData` as a type (Task 4 Step 1). The existing export at `useTimer.ts:22` is `export interface TimerStateData` — should import cleanly with `import type`.
- **`applyTimerState` semantic:** the setter must accept a `TimerStateData` (not wrap in `Timer`). The existing setter at `useTimer.ts` (added in the prior refactor) takes `TimerStateData` directly. Verify before Task 5 if uncertain.
- **Subscription load on Tasks page:** After Task 6, the `task:list_updated` IPC reload is global (mounted in `EventBus` from app start). Previously it was only active on the Tasks page. Net effect: same IPC volume (one reload per task mutation) but now happens even when the user is on Timer page. Acceptable — same as `fetchTimer` already being global.
