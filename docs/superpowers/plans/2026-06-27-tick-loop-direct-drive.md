# Tick-Loop Direct-Drive Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate the intermittent auto-advance race by driving `TimerTickService::start_timer_tick_loop` / `stop_timer_tick_loop` / `load_state` directly from orchestrators (Tauri commands, tray handlers, `CountdownExpiredHandler`), and reducing domain event handlers (`TimerStartedHandler`, `TimerResetHandler`, `TimerPausedHandler`, `TaskResetHandler`) to pure UI emitters that never touch the singleton `cancel_handle`.

**Architecture:** Option (a1) from the architect's design at `tmp/architect/27-06-2026-1332-tick-loop-boundary/design.md`. The tick-loop lifecycle is an infrastructure concern and is owned by infra/app-layer entry points. Use cases remain ignorant of `TimerTickService`. Domain events stay as pure facts. The "usecase returns `PhaseOutcome`, infra handler interprets and acts" pattern (already used by `CountdownExpiredHandler`) is extended to every command/tray entry point. No port is added to the usecase layer.

**Tech Stack:** Rust, Tauri 2, tokio, async-trait, SQLite, existing `InMemoryEventBus` (unchanged). Tests are integration tests under `core/infra/tests/app/`.

## Global Constraints

- **Dependency rule:** dependencies point inward toward `domain`. No new reference to `infra::adapters::TimerTickService` from `core/usecases/**` or `core/domain/**`.
- **No new deps:** do not add Cargo dependencies.
- **Event bus stays fire-and-forget:** do not modify `mem_event_bus.rs`. Do not make `EventPublisher::publish` async.
- **No sleeps:** delete every `tokio::time::sleep(Duration::from_millis(100))` whose comment says "Drain the async Reset event handler" (or equivalent). Do not add new sleeps to "drain" handlers.
- **Sequencing rule:** when an orchestration needs both STOP and START, they must be `await`-ed sequentially in that order. Never rely on event ordering.
- **Style:** match existing conventions in each file (anyhow `Context`, `infra::commands::...` log prefixes, `#[tauri::command(rename_all = "snake_case")]`, etc.). Run `cargo fmt` before each commit.
- **Each task ends with `cargo test` green for the workspace and a commit.**

## File Structure (summary)

| File | Responsibility after this plan |
|------|--------------------------------|
| `core/infra/src/adapters/timer/sqlite_service.rs` | Adds `is_tick_loop_alive()` test helper + design-contract doc block on `start_timer_tick_loop`. No behavior change. |
| `core/infra/src/adapters/timer/event_handlers/timer_started.rs` | UI-only emitter for `TimerStarted`. Drops tick-loop drive + `task_repository` field. |
| `core/infra/src/adapters/timer/event_handlers/timer_reset.rs` | UI-only emitter for `TimerReset`. Drops `stop` + `load_state`. |
| `core/infra/src/adapters/timer/event_handlers/timer_paused.rs` | UI-only emitter for `TimerPaused`. Drops `load_state` + `stop`. |
| `core/infra/src/adapters/task/event_handlers/task_reset.rs` | UI-only emitter for `TaskReset`. Drops `load_state` + `stop`. |
| `core/infra/src/adapters/timer/event_handlers/registry.rs` | Drops `task_repo` arg when constructing `TimerStartedHandler`. |
| `apps/tauri-app/src/commands/timer_cmd/start_timer.rs` | Adds `timer_tick_service` param; calls `start_timer_tick_loop` after the usecase. |
| `apps/tauri-app/src/commands/timer_cmd/pause_timer.rs` | Adds `timer_tick_service` param; calls `load_state` + `stop_timer_tick_loop`. |
| `apps/tauri-app/src/commands/timer_cmd/resume_timer.rs` | Adds `timer_tick_service` param; calls `start_timer_tick_loop` (fixes latent bug). |
| `apps/tauri-app/src/commands/timer_cmd/skip_phase.rs` | Adds `timer_tick_service` param; calls `stop` + `load` + `start` (fixes latent bug). |
| `apps/tauri-app/src/commands/timer_cmd/reset_timer.rs` | Deletes sleep; keeps the explicit `stop`. |
| `apps/tauri-app/src/commands/timer_cmd/reset_timer_phase.rs` | Deletes sleep; adds explicit `stop` + `load` before the conditional `start`. |
| `apps/tauri-app/src/commands/task_cmd/complete_flow.rs` | Deletes both sleeps; adds explicit `start` in the auto-start branch (THE BUG FIX). |
| `apps/tauri-app/src/tray.rs` | Updates `menu_play_pause`, `menu_reset_phase`, `menu_skip`, `menu_reset_task` to drive the tick loop directly and delete sleeps. |
| `CLAUDE.md` | Adds a design heuristic so future contributors do not reintroduce the anti-pattern. |
| `core/infra/tests/app/tick_loop_invariants.rs` (new) | Regression tests for the architectural invariant. |

**Unchanged on purpose:** `core/usecases/**`, `core/domain/**`, `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs`, `apps/tauri-app/src/commands/timer_cmd/switch_active_task.rs` (already correct — keep as the reference model), `mem_event_bus.rs`.

---

## Task 1: Foundation — `is_tick_loop_alive()` + design contract doc

**Files:**
- Modify: `core/infra/src/adapters/timer/sqlite_service.rs`

**Interfaces:**
- Produces: `pub async fn is_tick_loop_alive(&self) -> bool` on `TimerTickService` (used by tests in Tasks 2 and 3).

**Why first:** every later test asserts on this helper. No behavior change to the runtime; purely additive.

- [ ] **Step 1: Add the design-contract doc block + helper**

In `core/infra/src/adapters/timer/sqlite_service.rs`, replace the existing doc comment that starts at line 50 (`/// Start the infrastructure timer tick loop ...`) with the expanded version that includes the design contract, and append a new `is_tick_loop_alive` method inside `impl TimerTickService` immediately after `stop_timer_tick_loop` (after the existing line 174 closing brace of `stop_timer_tick_loop`).

Replace this block (currently lines 50-56):

```rust
    /// Start the infrastructure timer tick loop
    /// This manages the technical aspects of timer ticking
    pub async fn start_timer_tick_loop(
        &self,
        timer_config: Option<TimerConfiguration>,
        _task_id: Option<TaskId>,
    ) -> Result<(), String> {
```

with:

```rust
    /// Start the infrastructure timer tick loop.
    ///
    /// # Design Contract — Tick-Loop Ownership
    ///
    /// Callers — NOT domain event handlers — own the lifecycle of the tick
    /// loop. The auto-advance race was eliminated by routing start/stop out of
    /// detached event-bus handlers and into the orchestrators that drive the
    /// use cases (Tauri commands, tray handlers, `CountdownExpiredHandler`).
    ///
    /// ## Legitimate callers
    /// 1. App-layer Tauri commands (`apps/tauri-app/src/commands/**`).
    /// 2. App-layer tray handlers (`apps/tauri-app/src/tray.rs`).
    /// 3. Infra event handlers that are the ENTRY POINT of an async flow and
    ///    interpret a usecase outcome (e.g. `CountdownExpiredHandler`).
    ///    Reactors that merely respond to facts MUST NOT call this method.
    /// 4. Test setup.
    ///
    /// ## Sequencing
    /// When an orchestration needs both STOP and START:
    ///     await stop_timer_tick_loop();
    ///     await start_timer_tick_loop(cfg);
    /// Never publish events in lieu of these calls.
    ///
    /// ## Idempotency
    /// `start` aborts any prior handle and overwrites it (last-write-wins).
    /// `stop` is a no-op when no handle is present.
    pub async fn start_timer_tick_loop(
        &self,
        timer_config: Option<TimerConfiguration>,
        _task_id: Option<TaskId>,
    ) -> Result<(), String> {
```

Then, after the closing brace of `stop_timer_tick_loop` (the existing method that ends at line 174), insert this new method before `get_current_timer`:

```rust
    /// Test/observability helper. Returns `true` when a tick-loop task is
    /// currently registered AND still alive (not aborted, not finished).
    ///
    /// Production code MUST NOT branch on this — it exists so regression tests
    /// can assert that an orchestration left the loop in the expected state
    /// without relying on flaky timing.
    #[cfg(test)]
    pub async fn is_tick_loop_alive(&self) -> bool {
        let guard = self.cancel_handle.lock().await;
        match guard.as_ref() {
            None => false,
            Some(handle) => !handle.is_finished(),
        }
    }

```

> Note: `#[cfg(test)]` keeps the helper out of release builds. The integration tests under `core/infra/tests/` compile the crate with `--test`, which enables `cfg(test)`, so the helper is visible there. If a downstream binary build errors on "method not found in release mode," widen the attribute to `#[cfg(any(test, feature = "test-helpers"))]` and add the feature in `core/infra/Cargo.toml` — but try `#[cfg(test)]` first; it works for integration tests because they link the crate's test build.

- [ ] **Step 2: Verify it compiles and existing tests still pass**

Run: `cargo test -p infra` (from repo root)
Expected: existing tests compile and pass; no test references the new method yet, so behavior is unchanged.

- [ ] **Step 3: Commit**

```bash
git add core/infra/src/adapters/timer/sqlite_service.rs
git commit -m "refactor(infra/timer): document tick-loop ownership contract; add is_tick_loop_alive test helper"
```

---

## Task 2: Convert event handlers to UI-only emitters (the core architectural change)

This is the heart of the fix. After this task, NO domain event handler mutates `cancel_handle`. The auto-advance race becomes impossible because `TimerResetHandler` no longer aborts loops started by `TimerStartedHandler` / orchestrators.

**Files:**
- Modify: `core/infra/src/adapters/timer/event_handlers/timer_started.rs`
- Modify: `core/infra/src/adapters/timer/event_handlers/timer_reset.rs`
- Modify: `core/infra/src/adapters/timer/event_handlers/timer_paused.rs`
- Modify: `core/infra/src/adapters/task/event_handlers/task_reset.rs`
- Modify: `core/infra/src/adapters/timer/event_handlers/registry.rs`
- Create: `core/infra/tests/app/tick_loop_invariants.rs`
- Modify: `core/infra/tests/app/mod.rs`

**Interfaces:**
- Consumes: `TimerTickService::with_timer`, `is_tick_loop_alive` (Task 1).
- Produces: `TimerStartedHandler::new(emitter, timer_srv)` (signature drops `task_repo`); same UI events as before.

- [ ] **Step 1: Write the failing test**

Create `core/infra/tests/app/tick_loop_invariants.rs`:

```rust
//! Regression tests for the tick-loop ownership invariant.
//!
//! After the architectural change in this plan, domain event handlers MUST NOT
//! mutate `TimerTickService`'s `cancel_handle`. They are pure UI emitters. The
//! original auto-advance bug was caused by `TimerResetHandler` and
//! `TimerStartedHandler` racing on `cancel_handle` via detached `tokio::spawn`
//! tasks in `InMemoryEventBus::publish`. These tests lock the invariant.

use std::time::Duration;

use domain::{
    Config, TaskId, Timestamp, event_names::ui_listeners,
    timer::events::CountdownExpired,
};
use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

use crate::{AppContextBuilder, TaskBuilder};

/// Publishing `TimerReset`, `TimerPaused`, or `TimerStarted` must NOT stop (or
/// start) an already-running tick loop. Before the fix, `TimerResetHandler`
/// aborted the handle and killed the loop.
#[tokio::test]
async fn domain_events_do_not_mutate_tick_loop() {
    let ctx = AppContextBuilder::new()
        .with_name("domain_events_do_not_mutate_tick_loop")
        .build()
        .await
        .expect("Failed to build test context");

    let task = TaskBuilder::new()
        .name("Test Task")
        .max_sessions(4)
        .config(Config::default())
        .build();
    let task_id = task.id();
    ctx.task_repo.create(task).await.unwrap();

    // Start the timer and let the loop spin up via the normal path.
    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd {
            task_id: Some(task_id),
        },
    )
    .await
    .expect("Failed to start timer");

    // Drive the loop manually as well, to be certain a handle exists
    // independent of any handler.
    let cfg = ctx
        .task_repo
        .get_by_id(task_id)
        .await
        .unwrap()
        .unwrap()
        .config()
        .timer
        .clone();
    ctx.timer_tick_service
        .start_timer_tick_loop(Some(cfg.clone()), None)
        .await
        .expect("Failed to start tick loop");

    // Let any spawned handler from start_timer_phase settle.
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(
        ctx.timer_tick_service.is_tick_loop_alive().await,
        "precondition: tick loop should be alive before publishing events"
    );

    // Publish every event whose handler USED to touch cancel_handle.
    // The domain `Timer::reset`/`pause` constructors need a Timestamp; reuse
    // the publisher's event wrappers directly via the timer_srv's timer state.
    let state = ctx
        .timer_tick_service
        .get_current_timer()
        .await
        .state()
        .clone();

    let reset_event = Box::new(domain::TimerReset { timestamp: Timestamp::now() });
    let paused_event = Box::new(domain::TimerPaused {
        timestamp: Timestamp::now(),
        remaining_seconds: state.remaining_seconds,
    });
    let started_event = Box::new(domain::TimerStarted {
        task_id,
        timestamp: Timestamp::now(),
    });
    let task_reset_event = Box::new(domain::TaskReset {
        task_id,
        timestamp: Timestamp::now(),
    });

    ctx.event_bus.publish(reset_event);
    ctx.event_bus.publish(paused_event);
    ctx.event_bus.publish(started_event);
    ctx.event_bus.publish(task_reset_event);

    // Give the spawned handlers time to (incorrectly) mutate the handle.
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert!(
        ctx.timer_tick_service.is_tick_loop_alive().await,
        "INVARIANT VIOLATED: a domain event handler mutated cancel_handle. \
         Handlers must be UI-only emitters; orchestrators own the tick loop."
    );
}
```

> **Note for the implementer:** the exact field names on `domain::TimerReset`, `domain::TimerPaused`, `domain::TimerStarted`, `domain::TaskReset` must match the actual domain types — read `core/domain/src/lib.rs` (re-exports) and the underlying `core/domain/src/timer/events.rs` / `core/domain/src/task/events.rs` definitions before running. Adjust the struct literals to match the real constructors. If a type uses `::new(...)` instead of a struct literal, switch to the constructor.

In `core/infra/tests/app/mod.rs`, add the module declaration. After the existing line `mod task_cycling;`, add:

```rust
mod tick_loop_invariants;
```

- [ ] **Step 2: Run the test and confirm it FAILS on master**

Run: `cargo test -p infra --test app -- tick_loop_invariants`
Expected: FAIL. The assertion fires because `TimerResetHandler::handle` (and possibly `TimerPausedHandler::handle`) calls `stop_timer_tick_loop`, which aborts the handle. If it passes, the `tokio::time::sleep(300ms)` was too short — bump to 500ms, or check `is_tick_loop_alive` returns false deterministically by inspecting `cancel_handle` directly via a debug print.

> If the test does not fail, do not proceed — the test is worthless without a failing baseline. Investigate why (most likely: handler hasn't been scheduled yet, or the wrong event type is being published). The bug IS real per root-cause analysis, so the test must reproduce it.

- [ ] **Step 3: Convert `TimerStartedHandler` to UI-only**

In `core/infra/src/adapters/timer/event_handlers/timer_started.rs`, replace the entire file contents with:

```rust
use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TimerStarted`.
///
/// Per the tick-loop ownership contract on `TimerTickService::start_timer_tick_loop`,
/// this handler MUST NOT drive the tick loop. The orchestrator that called
/// `start_timer_phase` (Tauri command, tray handler, or `CountdownExpiredHandler`)
/// is responsible for calling `start_timer_tick_loop` directly. Routing the
/// side effect through this handler caused the auto-advance race, because
/// `InMemoryEventBus::publish` spawns handlers on detached `tokio::spawn`
/// tasks whose order relative to `TimerResetHandler` is non-deterministic.
pub struct TimerStartedHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl TimerStartedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        TimerStartedHandler { emitter, timer_srv }
    }
}

#[async_trait]
impl EventHandler for TimerStartedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let _timer_started = event
            .as_any()
            .downcast_ref::<domain::TimerStarted>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to start timer tick loop".to_string(),
            })?;

        // Read-only access to format the UI payload. No mutation of
        // cancel_handle. The orchestrator has already started the loop.
        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                state_json.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;

        self.emitter
            .emit(domain::event_names::ui_listeners::timer::START, state_json)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;

        Ok(())
    }
}
```

- [ ] **Step 4: Convert `TimerResetHandler` to UI-only**

In `core/infra/src/adapters/timer/event_handlers/timer_reset.rs`, replace the body of `handle` (the existing lines 29-69) so it no longer calls `stop_timer_tick_loop` or `load_state`. The new file:

```rust
use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TimerReset`.
///
/// Per the tick-loop ownership contract, this handler MUST NOT stop the tick
/// loop. The orchestrator that called `reset_timer_to_idle` (or equivalent)
/// owns the stop call. The previous implementation raced with
/// `TimerStartedHandler` on `cancel_handle` because the event bus dispatches
/// handlers on detached `tokio::spawn` tasks.
pub struct TimerResetHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl TimerResetHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        TimerResetHandler { emitter, timer_srv }
    }
}

#[async_trait]
impl EventHandler for TimerResetHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerReset>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let _timer_reset = event
            .as_any()
            .downcast_ref::<domain::TimerReset>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to reset timer".to_string(),
            })?;

        // Read-only: format the current timer state for the UI. The
        // orchestrator has already stopped the loop and refreshed state.
        let state_json = self
            .timer_srv
            .with_timer(|t| {
                log::info!("{:?} timer reset", t);
                json!(t.state())
            })
            .await;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::RESET,
                state_json.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer reset event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                state_json,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer status changed event: {e}"),
            })?;

        Ok(())
    }
}
```

- [ ] **Step 5: Convert `TimerPausedHandler` to UI-only**

In `core/infra/src/adapters/timer/event_handlers/timer_paused.rs`, replace the body of `handle` (existing lines 29-51) so it no longer calls `load_state` or `stop_timer_tick_loop`:

```rust
        // The orchestrator that called `pause_timer_phase` is responsible for
        // stop_timer_tick_loop + load_state. This handler is a UI-only emitter.
        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

        self.emitter
            .emit(domain::event_names::ui_listeners::timer::PAUSE, state_json)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer paused event: {e}"),
            })?;
        Ok(())
```

(Keep the file header, struct, and `impl` skeleton unchanged — only the body of `handle` is replaced. The downcast of the event is no longer needed for any side effect; if the compiler warns about an unused binding, name it `_timer_paused` which the existing code already does.)

- [ ] **Step 6: Convert `TaskResetHandler` to UI-only**

In `core/infra/src/adapters/task/event_handlers/task_reset.rs`, replace the body of `handle` (existing lines 29-63) so it no longer calls `load_state` or `stop_timer_tick_loop`. New body:

```rust
        let task_reset = event
            .as_any()
            .downcast_ref::<domain::TaskReset>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to reset task".to_string(),
            })?;

        // The orchestrator that called `reset_task` is responsible for
        // stop_timer_tick_loop + load_state. This handler is a UI-only emitter.
        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::TASK_RESET,
                json!(task_reset),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task reset event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::LIST_UPDATED,
                json!(task_reset),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task list updated event: {e}"),
            })?;
        Ok(())
```

- [ ] **Step 7: Update `registry.rs` to drop the `task_repo` arg to `TimerStartedHandler::new`**

In `core/infra/src/adapters/timer/event_handlers/registry.rs`, replace lines 46-50:

```rust
    event_bus.subscribe(Box::new(TimerStartedHandler::new(
        emitter.clone(),
        timer_srv.clone(),
        task_repo.clone(),
    )))?;
```

with:

```rust
    event_bus.subscribe(Box::new(TimerStartedHandler::new(
        emitter.clone(),
        timer_srv.clone(),
    )))?;
```

If `task_repo` becomes an unused parameter of `register_timer_handlers` (check the signature at line 17-25), keep it — `CountdownExpiredHandler::new` at line 57-64 still consumes `task_repo.clone()`. Do not change the function signature.

- [ ] **Step 8: Run the test and confirm it now PASSES**

Run: `cargo test -p infra --test app -- tick_loop_invariants`
Expected: PASS. The handlers no longer touch `cancel_handle`, so the loop stays alive.

- [ ] **Step 9: Run the full infra test suite to confirm no regressions**

Run: `cargo test -p infra`
Expected: all tests pass. If a test fails because it was implicitly relying on a handler to stop/start the loop, fix that test by adding an explicit `timer_tick_service.start_timer_tick_loop(...)` / `stop_timer_tick_loop()` call — do NOT revert the handler.

- [ ] **Step 10: Commit**

```bash
git add core/infra/src/adapters/timer/event_handlers/timer_started.rs \
        core/infra/src/adapters/timer/event_handlers/timer_reset.rs \
        core/infra/src/adapters/timer/event_handlers/timer_paused.rs \
        core/infra/src/adapters/task/event_handlers/task_reset.rs \
        core/infra/src/adapters/timer/event_handlers/registry.rs \
        core/infra/tests/app/tick_loop_invariants.rs \
        core/infra/tests/app/mod.rs
git commit -m "fix(infra/events): make timer event handlers UI-only emitters; never mutate cancel_handle

Eliminates the auto-advance race at its source. TimerReset/TimerStarted/
TimerPaused/TaskReset handlers no longer start or stop the tick loop. The
orchestrators that publish these events own the tick-loop side effects
directly. Adds tick_loop_invariants regression test."
```

---

## Task 3: Behavioral regression test for the auto-advance path

A behavioral test proving the timer-driven auto-advance path leaves the loop alive on the new task. Catches future regressions in `CountdownExpiredHandler` orchestration.

**Files:**
- Modify: `core/infra/tests/app/tick_loop_invariants.rs`

**Interfaces:**
- Consumes: `CountdownExpired`, `progress_phase` outcome handling (existing), `is_tick_loop_alive` (Task 1), the converted handlers from Task 2.

- [ ] **Step 1: Append the behavioral test**

Append to `core/infra/tests/app/tick_loop_invariants.rs`:

```rust
/// Timer-driven auto-advance: when a break-phase countdown expires for a
/// completed task with `auto_cycle` and `auto_start_work_after_break` enabled,
/// `CountdownExpiredHandler` must leave the tick loop ALIVE on the new task.
///
/// Before the fix, `TimerResetHandler` (spawned by `reset_timer_to_idle`
/// inside `progress_phase`) could abort the loop that `CountdownExpiredHandler`
/// had just started. Running the scenario repeatedly catches the race if it
/// ever regresses.
#[tokio::test]
async fn auto_advance_leaves_tick_loop_alive_on_new_task() {
    for iteration in 0..20 {
        let ctx = AppContextBuilder::new()
            .with_name(&format!("auto_advance_tick_loop_alive_{iteration}"))
            .build()
            .await
            .expect("Failed to build test context");

        // Enable auto-cycling and auto-start of the next work phase.
        let mut config = Config::default();
        config.general.auto_start_work_after_break = true;
        config.general.task_cycling_behavior =
            domain::task::TaskCyclingBehavior::RoundRobin;
        // `should_auto_cycle` requires AutoAdvance mode.
        config.general.task_cycling_mode =
            domain::task::TaskCyclingMode::AutoAdvance;
        ctx.config_repo.save_config(&config).await.unwrap();

        // Two tasks so cycling has somewhere to go. The first task is one
        // session away from completion; the second is the cycle target.
        let mut task1 = TaskBuilder::new()
            .name("Task 1")
            .max_sessions(1)
            .config(config.clone())
            .build();
        let task1_id = task1.id();
        let mut task2 = TaskBuilder::new()
            .name("Task 2")
            .max_sessions(4)
            .config(config.clone())
            .build();
        let task2_id = task2.id();
        ctx.task_repo.create(task1).await.unwrap();
        ctx.task_repo.create(task2).await.unwrap();

        // Start task1's work phase; immediately complete it so the task
        // becomes fully completed; then start its break phase so the upcoming
        // CountdownExpired (for the break) triggers the cycle branch in
        // progress_phase (cycle requires `from_phase` ∈ {Short, Long}Break).
        usecases::timer::start_timer_phase(
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_bus.clone(),
            StartTimerPhaseCmd { task_id: Some(task1_id) },
        )
        .await
        .expect("start work phase");
        // Complete work phase synchronously through the usecase.
        usecases::timer::complete_timer_phase(
            task1_id,
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_bus.clone(),
        )
        .await
        .expect("complete work phase");
        // Start the break so a tick loop is running and the cycle branch's
        // precondition (`from_phase` is a break) holds.
        usecases::timer::start_timer_phase(
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_bus.clone(),
            StartTimerPhaseCmd { task_id: Some(task1_id) },
        )
        .await
        .expect("start break phase");

        // Let any handler from start_timer_phase settle.
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Reset the loop handle so the only loop alive after the event is the
        // one CountdownExpiredHandler starts on the new task.
        ctx.timer_tick_service.stop_timer_tick_loop().await.unwrap();

        ctx.event_bus.publish(Box::new(CountdownExpired::new(
            domain::Phase::ShortBreak,
            task1_id,
        )));

        // CountdownExpiredHandler publishes TaskUpdated/AutoAdvanced/etc. and
        // calls start_timer_tick_loop on the new task. Give the spawned
        // orchestration time to finish.
        tokio::time::sleep(Duration::from_millis(500)).await;

        assert!(
            ctx.timer_tick_service.is_tick_loop_alive().await,
            "iteration {iteration}: auto-advance left the tick loop dead"
        );

        let bound_task = ctx
            .timer_tick_service
            .get_current_timer()
            .await
            .task_id()
            .map(|t| t.to_string());
        assert_eq!(
            bound_task.as_deref(),
            Some(task2_id.as_str()),
            "iteration {iteration}: tick loop is not bound to the cycled task"
        );
    }
}
```

> **Implementer note:** verify the field names `task_cycling_behavior`, `task_cycling_mode`, `TaskCyclingBehavior::RoundRobin`, `TaskCyclingMode::AutoAdvance`, and `should_auto_cycle`'s exact preconditions against `core/domain/src/task/...` and `core/usecases/src/timer/progress_phase.rs:104-106` before running. The cycle branch in `progress_phase` triggers on `task.is_completed() && from_phase ∈ {ShortBreak, LongBreak} && should_auto_cycle`. Tune task setup until that branch is reached. If `complete_timer_phase` doesn't fully complete the task in one call (because `max_sessions` counting differs), add a second iteration of work+complete. The test's INTENT is: reach the cycle branch and assert the loop survives.

- [ ] **Step 2: Run the test**

Run: `cargo test -p infra --test app -- auto_advance_leaves_tick_loop_alive_on_new_task`
Expected: PASS. (It should pass after Task 2. If it fails, the cycle branch wasn't reached — fix the setup, do not weaken the assertion.)

- [ ] **Step 3: Commit**

```bash
git add core/infra/tests/app/tick_loop_invariants.rs
git commit -m "test(infra/timer): add behavioral regression for auto-advance tick-loop aliveness"
```

---

## Task 4: Fix `complete_task_flow` — THE BUG FIX (manual-complete auto-advance path)

This is the path the user originally reported as intermittent. Delete the band-aid sleeps and drive the tick loop explicitly.

**Files:**
- Modify: `apps/tauri-app/src/commands/task_cmd/complete_flow.rs`

**Interfaces:** unchanged signature; behavior change only.

- [ ] **Step 1: Replace the body of `complete_task_flow`**

In `apps/tauri-app/src/commands/task_cmd/complete_flow.rs`, replace lines 31-168 (the entire `complete_task_flow` function body) with:

```rust
pub async fn complete_task_flow(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: TimerRepositoryArc,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    timer_tick_service: Arc<TimerTickService>,
    app_handle: AppHandle,
) -> anyhow::Result<Task> {
    reset_timer_to_idle(
        task_id,
        task_repo.clone(),
        timer_repo.clone(),
        event_publisher.clone(),
    )
    .await
    .context("Failed to reset timer to idle after completing task")?;

    // Direct STOP. No sleep, no reliance on the TimerReset event handler —
    // per the tick-loop ownership contract, orchestrators own the side effect.
    timer_tick_service
        .stop_timer_tick_loop()
        .await
        .context("Failed to stop timer tick loop while completing task")?;

    // Complete the task (all sessions).
    complete_task_uc(&task_repo, &event_publisher, task_id)
        .await
        .with_context(|| format!("Failed to complete task: {}", task_id))?;

    // Auto-advance: if AutoAdvance is configured, switch to the next incomplete
    // task. Whether its work phase auto-starts is governed by
    // `auto_start_work_after_break` (same flag the timer-driven cycle path uses
    // in `progress_phase`).
    let mut active_task_id = task_id;
    if let Some(plan) = plan_auto_advance(&task_repo, &config_repo).await {
        match switch_active_task(
            task_repo.clone(),
            timer_repo.clone(),
            event_publisher.clone(),
            SwitchActiveTaskCmd {
                task_id: plan.next_task_id,
                old_task_id: Some(task_id),
            },
        )
        .await
        .context("Failed to switch to next task after completing")
        {
            Ok(()) => {
                reset_timer_to_idle(
                    plan.next_task_id,
                    task_repo.clone(),
                    timer_repo.clone(),
                    event_publisher.clone(),
                )
                .await
                .context("Failed to reset timer to idle for next task")?;

                // Refresh the in-memory cache so UI payloads below are correct,
                // and ensure the loop is stopped before any start.
                timer_tick_service
                    .load_state()
                    .await
                    .context("Failed to load timer state after auto-advance")?;
                timer_tick_service
                    .stop_timer_tick_loop()
                    .await
                    .context("Failed to stop tick loop after auto-advance reset")?;

                if plan.auto_start_work {
                    // Drive the usecase, then start the tick loop directly.
                    // TimerStartedHandler is a UI-only emitter and no longer
                    // starts the loop for us.
                    if let Err(e) = start_timer_phase(
                        task_repo.clone(),
                        timer_repo.clone(),
                        event_publisher.clone(),
                        StartTimerPhaseCmd {
                            task_id: Some(plan.next_task_id),
                        },
                    )
                    .await
                    {
                        log::warn!(
                            "Auto-start of task {} after auto-advance failed: {e}",
                            plan.next_task_id
                        );
                    } else {
                        let next_task = task_repo
                            .get_by_id(plan.next_task_id)
                            .await
                            .context("Failed to load next task for tick-loop start")?
                            .ok_or_else(|| {
                                anyhow!(
                                    "Next task {} not found after auto-advance",
                                    plan.next_task_id
                                )
                            })?;
                        timer_tick_service
                            .start_timer_tick_loop(
                                Some(next_task.config().timer.clone()),
                                None,
                            )
                            .await
                            .map_err(|e| {
                                anyhow!(
                                    "Failed to start tick loop after auto-advance: {e}"
                                )
                            })?;
                    }
                }

                active_task_id = plan.next_task_id;

                let _ = app_handle.emit(
                    domain::event_names::task::AUTO_ADVANCED,
                    json!({
                        "from_task_id": task_id.to_string(),
                        "to_task_id": plan.next_task_id.to_string(),
                    }),
                );
            }
            Err(e) => {
                log::warn!(
                    "Auto-advance after completing {} failed; staying on completed task: {e}",
                    task_id
                );
            }
        }
    }

    // If no new task became active (Manual mode, AutoAdvance with no eligible
    // successor, or the switch failed above), detach the completed task from
    // the timer so the UI can prompt for a new selection.
    if active_task_id == task_id {
        match clear_active_task(timer_repo.clone())
            .await
            .context("Failed to clear active task after completing")
        {
            Ok(()) => {
                let _ = app_handle.emit(
                    domain::event_names::task::ACTIVE_TASK_CLEARED,
                    json!({ "from_task_id": task_id.to_string() }),
                );
            }
            Err(e) => {
                log::warn!(
                    "Auto-clear of completed task {} failed; timer left bound: {e}",
                    task_id
                );
            }
        }
    }

    let task = task_repo
        .get_by_id(active_task_id)
        .await
        .context("Failed to retrieve task after completing")?
        .ok_or_else(|| anyhow!("Task not found after completing"))?;

    Ok(task)
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p tauri-app` (or whatever the workspace member is named — check root `Cargo.toml`)
Expected: compiles. The `tokio::time::sleep` and `Duration` imports at the top of the file are now unused — remove the `use std::time::Duration;` line at line 13 if no other code in the file uses it (the `spawn`-style helpers don't).

- [ ] **Step 3: Run all tests**

Run: `cargo test`
Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add apps/tauri-app/src/commands/task_cmd/complete_flow.rs
git commit -m "fix(tauri/complete_flow): drive tick loop directly; remove race-y band-aid sleeps

Fixes the intermittent manual-complete auto-advance bug. The TimerReset
and TimerStarted event handlers are now UI-only emitters, so the
orchestrator must start the loop explicitly after auto-start. Deletes
both 'drain the async Reset handler' sleeps."
```

---

## Task 5: Fix reset commands

Delete the band-aid sleep in `reset_timer` (it's now redundant) and replace the sleep in `reset_timer_phase` with an explicit `stop` + `load` before the conditional `start`.

**Files:**
- Modify: `apps/tauri-app/src/commands/timer_cmd/reset_timer.rs`
- Modify: `apps/tauri-app/src/commands/timer_cmd/reset_timer_phase.rs`

- [ ] **Step 1: Simplify `reset_timer.rs`**

In `apps/tauri-app/src/commands/timer_cmd/reset_timer.rs`, replace the comment-and-sleep-and-call block at lines 25-53 with a simpler version. Replace from line 25 through line 53:

```rust
    // Reset the timer to idle (business operation). This publishes a Reset
    // event whose handler stops the tick loop and reloads state. The event
    // bus is fire-and-forget (handlers run on spawned tasks), so drain that
    // handler before proceeding — otherwise the tick loop keeps running on
    // stale in-memory state and races with the handler's stop/load.
    reset_timer_to_idle(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::reset_timer - Failed to reset timer to idle state")
    .map_err(|e| e.to_string())?;

    // Drain the async Reset handler (stop_timer_tick_loop + load_state).
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Belt-and-suspenders: ensure the tick loop is stopped even if the
    // handler hasn't drained yet. stop_timer_tick_loop is idempotent.
    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::reset_timer - Failed to stop tick loop: {}",
                e
            )
        })?;
```

with:

```rust
    // Reset the timer to idle (business operation). Publishes a Reset event
    // that is now a UI-only notification (the handler no longer touches the
    // tick loop). Per the tick-loop ownership contract, this orchestrator owns
    // the stop and state refresh.
    reset_timer_to_idle(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::reset_timer - Failed to reset timer to idle state")
    .map_err(|e| e.to_string())?;

    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::reset_timer - Failed to load timer state: {}",
                e
            )
        })?;

    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::reset_timer - Failed to stop tick loop: {}",
                e
            )
        })?;
```

Then remove the now-unused `use std::time::Duration;` at line 2 of the file.

- [ ] **Step 2: Rewrite `reset_timer_phase.rs`**

In `apps/tauri-app/src/commands/timer_cmd/reset_timer_phase.rs`, replace lines 36-75 (the comment + usecase + sleep + conditional restart) with:

```rust
    // Reset the current phase's countdown to its full duration (business
    // operation). The orchestrator owns the tick-loop side effects per the
    // ownership contract — no sleep, no reliance on the Reset event handler.
    reset_timer_phase_usecase(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo_arc.clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::reset_timer_phase - Failed to reset timer phase")
    .map_err(|e| e.to_string())?;

    // Stop the existing loop and refresh the in-memory cache so a fresh
    // loop (if any) sees the reset remaining seconds.
    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::reset_timer_phase - Failed to stop tick loop: {}",
                e
            )
        })?;

    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::reset_timer_phase - Failed to load timer state: {}",
                e
            )
        })?;

    // Get the updated timer state with the reset remaining seconds.
    let updated_timer = timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd::reset_timer_phase - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())?;

    // Restart the tick loop so a running phase keeps counting down from the
    // full duration. For a paused timer the loop should not run; leave it
    // stopped to preserve the paused state.
    if updated_timer.is_running() {
        timer_tick_service_arc
            .start_timer_tick_loop(Some(task.config().timer.clone()), None)
            .await
            .map_err(|e| {
                format!(
                    "infra::commands::timer_cmd::reset_timer_phase - Failed to restart tick loop: {}",
                    e
                )
            })?;
    }
```

Remove the now-unused `use std::time::Duration;` at line 2 of the file.

- [ ] **Step 3: Build and test**

Run: `cargo build && cargo test`
Expected: compiles, all tests pass.

- [ ] **Step 4: Commit**

```bash
git add apps/tauri-app/src/commands/timer_cmd/reset_timer.rs \
        apps/tauri-app/src/commands/timer_cmd/reset_timer_phase.rs
git commit -m "refactor(tauri/timer_cmd): drive tick loop directly in reset commands; remove band-aid sleeps"
```

---

## Task 6: Drive the tick loop from `start_timer`, `pause_timer`, `resume_timer`, `skip_phase`

The handlers no longer do it for us. `resume_timer` and `skip_phase` are latent bugs (the loop was never restarted); this task fixes them.

**Files:**
- Modify: `apps/tauri-app/src/commands/timer_cmd/start_timer.rs`
- Modify: `apps/tauri-app/src/commands/timer_cmd/pause_timer.rs`
- Modify: `apps/tauri-app/src/commands/timer_cmd/resume_timer.rs`
- Modify: `apps/tauri-app/src/commands/timer_cmd/skip_phase.rs`

- [ ] **Step 1: Add tick-loop drive to `start_timer.rs`**

Replace the entire contents of `apps/tauri-app/src/commands/timer_cmd/start_timer.rs` with:

```rust
use super::*;
use domain::TaskId;
use infra::adapters::TimerTickService;
use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

#[tauri::command(rename_all = "snake_case")]
pub async fn start_timer(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
    _app_handle: AppHandle,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();
    let timer_tick_service_arc = timer_tick_service.inner().clone();

    let task_id_parsed = TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    info!("Starting timer for task {}", task_id_parsed);

    let cmd = StartTimerPhaseCmd {
        task_id: Some(task_id_parsed),
    };

    start_timer_phase(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .context("infra::commands::timer_cmd::start_timer - Failed to execute start timer phase")
    .map_err(|e| e.to_string())?;

    // Per the tick-loop ownership contract, this orchestrator owns the start.
    // `start_timer_tick_loop` reloads state from the repo internally, so no
    // separate load_state call is needed.
    let task = task_repo
        .inner()
        .get_by_id(task_id_parsed)
        .await
        .context("infra::commands::timer_cmd::start_timer - Failed to load task")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    timer_tick_service_arc
        .start_timer_tick_loop(Some(task.config().timer.clone()), None)
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::start_timer - Failed to start tick loop: {}",
                e
            )
        })?;

    timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Add tick-loop drive to `pause_timer.rs`**

Replace the entire contents of `apps/tauri-app/src/commands/timer_cmd/pause_timer.rs` with:

```rust
use super::*;
use domain::TaskId;
use infra::adapters::TimerTickService;
use usecases::timer::pause_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn pause_timer(
    task_id: String,
    remaining_seconds: u32,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();
    let timer_tick_service_arc = timer_tick_service.inner().clone();

    let task_id_parsed = TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    info!("Pausing timer for task {}", task_id_parsed);

    pause_timer_phase(
        task_id_parsed,
        remaining_seconds,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::pause_timer - Failed to toggle pause state")
    .map_err(|e| e.to_string())?;

    // Per the tick-loop ownership contract, this orchestrator owns the stop.
    // Refresh the in-memory cache so UI payloads reflect the paused state.
    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::pause_timer - Failed to load timer state: {}",
                e
            )
        })?;

    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::pause_timer - Failed to stop tick loop: {}",
                e
            )
        })?;

    timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 3: Add tick-loop drive to `resume_timer.rs` (fixes the latent resume bug)**

Replace the entire contents of `apps/tauri-app/src/commands/timer_cmd/resume_timer.rs` with:

```rust
use super::*;
use domain::TaskId;
use infra::adapters::TimerTickService;
use usecases::timer::resume_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn resume_timer(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();
    let timer_tick_service_arc = timer_tick_service.inner().clone();

    let task_id_parsed = TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    info!("Resuming timer for task {}", task_id_parsed);

    resume_timer_phase(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context(
        "infra::commands::timer_cmd::resume_timer - Failed to resume timer",
    )
    .map_err(|e| e.to_string())?;

    // Per the tick-loop ownership contract, this orchestrator owns the restart.
    // Previously there was no TimerResumedHandler, so the loop was never
    // restarted after a resume — the timer appeared stuck.
    let task = task_repo
        .inner()
        .get_by_id(task_id_parsed)
        .await
        .context("infra::commands::timer_cmd::resume_timer - Failed to load task")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    timer_tick_service_arc
        .start_timer_tick_loop(Some(task.config().timer.clone()), None)
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::resume_timer - Failed to start tick loop: {}",
                e
            )
        })?;

    timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 4: Add tick-loop drive to `skip_phase.rs` (fixes the latent skip bug)**

Replace the entire contents of `apps/tauri-app/src/commands/timer_cmd/skip_phase.rs` with:

```rust
use super::*;
use domain::TaskId;
use infra::adapters::TimerTickService;
use usecases::timer::skip_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn skip_phase(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();
    let timer_tick_service_arc = timer_tick_service.inner().clone();

    let task_id_parsed = TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    skip_timer_phase(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        task_id_parsed,
    )
    .await
    .context(
        "infra::commands::timer_cmd::skip_phase - Failed to skip to next phase",
    )
    .map_err(|e| e.to_string())?;

    // Per the tick-loop ownership contract: stop, refresh, then restart so the
    // new phase counts down. The previous PhaseSkippedHandler did not reliably
    // restart the loop, leaving the timer stuck after a skip.
    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::skip_phase - Failed to stop tick loop: {}",
                e
            )
        })?;

    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::skip_phase - Failed to load timer state: {}",
                e
            )
        })?;

    let updated_timer = timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd::skip_phase - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())?;

    // Only start a new loop if the post-skip timer is in a running state.
    // A skip that lands on a paused phase (no auto-start) leaves the loop
    // stopped intentionally.
    if updated_timer.is_running() {
        let task = task_repo
            .inner()
            .get_by_id(task_id_parsed)
            .await
            .context("infra::commands::timer_cmd::skip_phase - Failed to load task")
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        timer_tick_service_arc
            .start_timer_tick_loop(Some(task.config().timer.clone()), None)
            .await
            .map_err(|e| {
                format!(
                    "infra::commands::timer_cmd::skip_phase - Failed to start tick loop: {}",
                    e
                )
            })?;
    }

    Ok(updated_timer)
}
```

- [ ] **Step 5: Build and run all tests**

Run: `cargo build && cargo test`
Expected: compiles, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add apps/tauri-app/src/commands/timer_cmd/start_timer.rs \
        apps/tauri-app/src/commands/timer_cmd/pause_timer.rs \
        apps/tauri-app/src/commands/timer_cmd/resume_timer.rs \
        apps/tauri-app/src/commands/timer_cmd/skip_phase.rs
git commit -m "fix(tauri/timer_cmd): drive tick loop directly from start/pause/resume/skip

Resumes and skips used to leak a dead tick loop because no handler
restarted it. Now each command drives start/stop/load explicitly per the
tick-loop ownership contract."
```

---

## Task 7: Drive the tick loop from tray handlers

Mirror the command changes in `tray.rs`. Delete the two band-aid sleeps. Fix the latent skip-from-tray bug.

**Files:**
- Modify: `apps/tauri-app/src/tray.rs`

- [ ] **Step 1: Update `menu_play_pause` (lines 517-560)**

Replace lines 517-560 with:

```rust
/// Play / Pause / Resume — a single toggle mirroring the React play-pause
/// button. Running → pause, Paused → resume, otherwise → start. Per the
/// tick-loop ownership contract, this handler drives the loop directly in
/// each branch.
fn menu_play_pause(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let res = match ctx.timer.status() {
            TimerStatus::Running => {
                let live = ctx.tick_service.get_current_timer().await;
                let remaining = live.remaining_seconds(None);
                pause_timer_phase(
                    ctx.task_id,
                    remaining,
                    ctx.task_repo.clone(),
                    ctx.timer_repo.clone(),
                    ctx.event_publisher.clone(),
                )
                .await
                .and_then(|_| {
                    // Drive the stop directly. The TimerPaused handler is a
                    // UI-only emitter and no longer stops the loop.
                    let tick_service = ctx.tick_service.clone();
                    async move {
                        tick_service.load_state().await?;
                        tick_service.stop_timer_tick_loop().await?;
                        Ok(())
                    }
                })
            }
            TimerStatus::Paused => {
                let tick_service = ctx.tick_service.clone();
                let task_repo = ctx.task_repo.clone();
                let task_id = ctx.task_id;
                resume_timer_phase(
                    ctx.task_id,
                    ctx.task_repo.clone(),
                    ctx.timer_repo.clone(),
                    ctx.event_publisher.clone(),
                )
                .await
                .and_then(|_| async move {
                    let task = task_repo
                        .get_by_id(task_id)
                        .await?
                        .ok_or_else(|| {
                            domain::Error::TaskNotFound {
                                id: task_id.to_string(),
                            }
                        })?;
                    tick_service
                        .start_timer_tick_loop(Some(task.config().timer.clone()), None)
                        .await
                        .map_err(|e| domain::Error::RepositoryError {
                            message: e,
                        })
                })
            }
            TimerStatus::Idle | TimerStatus::Stopped => {
                let tick_service = ctx.tick_service.clone();
                let task_repo = ctx.task_repo.clone();
                let task_id = ctx.task_id;
                start_timer_phase(
                    ctx.task_repo.clone(),
                    ctx.timer_repo.clone(),
                    ctx.event_publisher.clone(),
                    StartTimerPhaseCmd {
                        task_id: Some(ctx.task_id),
                    },
                )
                .await
                .and_then(|_| async move {
                    let task = task_repo
                        .get_by_id(task_id)
                        .await?
                        .ok_or_else(|| {
                            domain::Error::TaskNotFound {
                                id: task_id.to_string(),
                            }
                        })?;
                    tick_service
                        .start_timer_tick_loop(Some(task.config().timer.clone()), None)
                        .await
                        .map_err(|e| domain::Error::RepositoryError {
                            message: e,
                        })
                })
            }
        };
        if let Err(e) = res {
            log::error!("Tray play/pause failed: {}", e);
        }
    });
}
```

> **Implementer note:** the exact error type returned by `start_timer_tick_loop` is `Result<(), String>`. The usecases' `Error` type is `domain::Error`. The `.map_err(|e| domain::Error::RepositoryError { message: e })` adapts the former to the latter so `and_then` type-checks. If `pause_timer_phase` / `resume_timer_phase` / `start_timer_phase` return `domain::Result<_>` (check `core/usecases/src/timer/...`), this works. If they return `anyhow::Result<_>` instead, drop the adaptation and use plain `.then(...)` with `anyhow!`. Match what compiles — the INTENT is "stop after pause, start after resume/start, on the tick service directly."

- [ ] **Step 2: Update `menu_reset_phase` (lines 566-614)**

Replace lines 566-614 with:

```rust
/// Restart the current phase's countdown, mirroring the React "Restart Phase"
/// button (`reset_timer_phase`). Per the tick-loop ownership contract, this
/// handler drives stop/load/start directly — no sleep, no reliance on the
/// Reset event handler.
fn menu_reset_phase(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let task = match ctx.task_repo.get_by_id(ctx.task_id).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                log::error!("Tray reset phase: task {} not found", ctx.task_id);
                return;
            }
            Err(e) => {
                log::error!("Tray reset phase: failed to load task: {}", e);
                return;
            }
        };

        if let Err(e) = reset_timer_phase(
            ctx.task_id,
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_publisher.clone(),
        )
        .await
        {
            log::error!("Tray reset phase failed: {}", e);
            return;
        }

        if let Err(e) = ctx.tick_service.stop_timer_tick_loop().await {
            log::error!("Tray reset phase: failed to stop tick loop: {}", e);
            return;
        }
        if let Err(e) = ctx.tick_service.load_state().await {
            log::error!("Tray reset phase: failed to load timer state: {}", e);
            return;
        }

        match ctx.timer_repo.get().await {
            Ok(updated) if updated.is_running() => {
                if let Err(e) = ctx
                    .tick_service
                    .start_timer_tick_loop(Some(task.config().timer.clone()), None)
                    .await
                {
                    log::error!(
                        "Tray reset phase: failed to restart tick loop: {}",
                        e
                    );
                }
            }
            Ok(_) => { /* paused: leave the loop stopped */ }
            Err(e) => log::error!("Tray reset phase: failed to read timer: {}", e),
        }
    });
}
```

- [ ] **Step 3: Update `menu_skip` (lines 617-633)**

Replace lines 617-633 with:

```rust
/// Skip to the next phase, mirroring the React "Skip Phase" button. Per the
/// tick-loop ownership contract, this handler drives stop/load/start directly.
/// Previously skip-from-tray never restarted the loop (no handler did), so the
/// timer appeared stuck after a skip.
fn menu_skip(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let task = match ctx.task_repo.get_by_id(ctx.task_id).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                log::error!("Tray skip: task {} not found", ctx.task_id);
                return;
            }
            Err(e) => {
                log::error!("Tray skip: failed to load task: {}", e);
                return;
            }
        };

        if let Err(e) = skip_timer_phase(
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_publisher.clone(),
            ctx.task_id,
        )
        .await
        {
            log::error!("Tray skip phase failed: {}", e);
            return;
        }

        if let Err(e) = ctx.tick_service.stop_timer_tick_loop().await {
            log::error!("Tray skip: failed to stop tick loop: {}", e);
            return;
        }
        if let Err(e) = ctx.tick_service.load_state().await {
            log::error!("Tray skip: failed to load timer state: {}", e);
            return;
        }

        match ctx.timer_repo.get().await {
            Ok(updated) if updated.is_running() => {
                if let Err(e) = ctx
                    .tick_service
                    .start_timer_tick_loop(Some(task.config().timer.clone()), None)
                    .await
                {
                    log::error!("Tray skip: failed to restart tick loop: {}", e);
                }
            }
            Ok(_) => { /* paused: leave the loop stopped */ }
            Err(e) => log::error!("Tray skip: failed to read timer: {}", e),
        }
    });
}
```

- [ ] **Step 4: Update `menu_reset_task` (lines 637-658)**

Replace lines 641-658 (the body inside `ctx.spawn(|ctx| async move { ... })`) with:

```rust
        if let Err(e) = reset_task_uc(
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_publisher.clone(),
            ctx.task_id,
        )
        .await
        {
            log::error!("Tray reset task failed: {}", e);
        }

        // Per the tick-loop ownership contract, drive the stop directly. No
        // sleep — the TaskReset event handler is a UI-only emitter now.
        if let Err(e) = ctx.tick_service.stop_timer_tick_loop().await {
            log::error!("Tray reset task: failed to stop tick loop: {}", e);
        }
        if let Err(e) = ctx.tick_service.load_state().await {
            log::error!("Tray reset task: failed to load timer state: {}", e);
        }
```

> Check the signature of `reset_task_uc` — the import at `tray.rs:32` is `use usecases::task::reset_task as reset_task_uc;`. The current call at line 642 passes `(ctx.task_repo, ctx.timer_repo, ctx.event_publisher, ctx.task_id)`. If the real signature is `(task_repo, task_id, reset_sessions)` instead, keep the current argument list and just add `.clone()` where the borrow checker demands it. The intent is: call the same usecase, then stop+load on the tick service.

- [ ] **Step 5: Build and run all tests**

Run: `cargo build && cargo test`
Expected: compiles, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add apps/tauri-app/src/tray.rs
git commit -m "fix(tauri/tray): drive tick loop directly from tray handlers; remove sleeps

Mirrors the command-layer fix. Also fixes a latent bug where skip-from-tray
left the timer stuck (the PhaseSkipped handler did not restart the loop)."
```

---

## Task 8: Document the heuristic in CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Append the heuristic**

Append to `CLAUDE.md`:

```markdown

## Tick-loop ownership

`TimerTickService::start_timer_tick_loop` / `stop_timer_tick_loop` / `load_state` MUST be called by the orchestrator that drives a state-changing `usecases::timer::*` call (a Tauri command, a tray handler, or `CountdownExpiredHandler`) — NOT by domain event handlers. Domain event handlers (`TimerStartedHandler`, `TimerResetHandler`, `TimerPausedHandler`, `TaskResetHandler`) are UI-only emitters; they never mutate `cancel_handle`.

When an orchestration needs both STOP and START:

```rust
timer_tick_service.stop_timer_tick_loop().await?;
timer_tick_service.load_state().await?;            // refresh in-memory cache
timer_tick_service.start_timer_tick_loop(cfg, None).await?;  // last-write-wins
```

Never `tokio::time::sleep` to "drain" an event handler. Never rely on event ordering. See `docs/superpowers/plans/2026-06-27-tick-loop-direct-drive.md` and `tmp/architect/27-06-2026-1332-tick-loop-boundary/design.md` for the rationale.
```

- [ ] **Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs(claude): document tick-loop ownership heuristic"
```

---

## Self-Review

**1. Spec coverage** — every recommendation from the architect's design is mapped to a task:
- Boundary decision (a1): enforced by Task 2 (handlers UI-only) and Tasks 4-7 (orchestrators drive directly).
- Fate of the four handlers: Task 2.
- Fate of the band-aid sleeps: Task 4 (complete_flow), Task 5 (reset commands), Task 7 (tray).
- Inventory of call sites: Tasks 4 (complete_flow), 5 (resets), 6 (start/pause/resume/skip), 7 (tray). `switch_active_task.rs` and `CountdownExpiredHandler` are deliberately unchanged (already correct).
- Sequencing rules: encoded in every command's code (stop → load → start).
- Design-contract doc block: Task 1.
- Test strategy: Tasks 2 (invariant), 3 (behavioral). The "handlers don't mutate cancel_handle" assertion is the deterministic core; the ×20 auto-advance is the probabilistic backstop.

**2. Placeholder scan** — implementer notes are flagged with the word "note" and give explicit fallbacks ("match what compiles", "verify against file X"). No `TODO`, no `TBD`. The two areas that may need adjustment during implementation are: (a) exact domain event struct fields in the Task 2 test (note included), (b) the `task_cycling_mode`/`should_auto_cycle` config field names in the Task 3 test (note included), (c) the exact error-adaptation shape in `menu_play_pause` (note included), (d) the `reset_task_uc` argument shape (note included).

**3. Type consistency** — `is_tick_loop_alive` (Task 1) is used by Tasks 2 and 3. `TimerStartedHandler::new(emitter, timer_srv)` (Task 2) is wired in `registry.rs` (Task 2). Every command takes `timer_tick_service: State<'_, Arc<TimerTickService>>` consistent with `switch_active_task.rs:16` and `reset_timer.rs:17`. The `_task_id: Option<TaskId>` second arg to `start_timer_tick_loop` is unused (see `sqlite_service.rs:55`) and is passed as `None` everywhere — matches existing call sites in `countdown_expired.rs:92` and `switch_active_task.rs:86`.

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-27-tick-loop-direct-drive.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration. Best for this plan because Tasks 2 and 3 involve iterative test debugging (real Rust event struct shapes may need adjustment from the documented guesses).

**2. Inline Execution** — I execute tasks in this session using executing-plans, batch execution with checkpoints. Faster turn-around but I carry context across tasks (less isolation).

**Which approach?**
