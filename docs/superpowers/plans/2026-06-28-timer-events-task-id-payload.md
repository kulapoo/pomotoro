# Timer UI events: include `task_id` in payload — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the four timer UI events (`timer:timer_started`, `timer:timer_paused`, `timer:timer_reset`, `timer:timer_resumed`) emit `{ task_id, state }` payloads so the React store can populate `timer.task_id` directly, and turn the orphaned `timer:timer_resumed` into a first-class event with its own domain semantics.

**Architecture:** A new `Resumed` domain event replaces the current misuse of `Started` in `transitions::resume()`. Each timer UI handler switches from emitting bare `state` JSON to emitting a `{ task_id, state }` JSON object built from the domain event's `task_id` plus the timer's current `state()`. Audio and notification adapters gain parallel `TimerResumedAudioHandler` / `TimerResumedNotificationHandler` structs that subscribe to `Resumed` and delegate to shared helper functions extracted from the existing `TimerStarted*` handlers. The React frontend switches its four listeners from `applyTimerState` (preserves task_id) to `applyTimer` (replaces the whole timer).

**Tech Stack:** Rust (domain, infra, usecases — `async-trait`, `serde_json`, `chrono`), React + TypeScript + Zustand + Tauri APIs.

## Global Constraints

- **Domain event pattern:** every new event follows the `Event` trait pattern used by `Started` (see `core/domain/src/timer/events/timer_started.rs`). `event_type()` returns a string matching the struct's intent name.
- **Handler pattern:** every new infra handler follows `TimerStartedHandler` exactly — read-only access via `TimerTickService::with_timer`, never mutates `cancel_handle`. See the tick-loop ownership contract in `CLAUDE.md` and `docs/superpowers/plans/2026-06-27-tick-loop-direct-drive.md`.
- **UI payload shape:** `{ "task_id": "<uuid>", "state": <TimerStateData> }` — matches the React `Timer` interface in `apps/react-ui/src/pages/timer/useTimer.ts:33`.
- **No emojis in code or commits.**
- **Commit message style:** match existing repo (`cargo fmt --check` and `cargo clippy` must pass — a pre-commit hook runs both, see `core/infra/tests/app/timer.rs:80` for the existing pattern).
- **TDD:** every code task writes a failing test first, runs it to confirm failure, then implements.
- **Type names:** the new domain event is `Resumed` (re-exported as `TimerResumed` from `domain`). The new handlers are `TimerResumedHandler` (infra/timer), `TimerResumedAudioHandler` (infra/audio), `TimerResumedNotificationHandler` (infra/notifications).

## File Structure

**Domain (new + modified):**
- Create `core/domain/src/timer/events/timer_resumed.rs` — new `Resumed` event struct + `Event` impl.
- Modify `core/domain/src/timer/events/mod.rs` — module + re-export declaration + tests.
- Modify `core/domain/src/lib.rs` — re-export `Resumed as TimerResumed`.
- Modify `core/domain/src/timer/transitions.rs` — `resume()` emits `Resumed`.

**Infra / timer handlers (new + modified):**
- Create `core/infra/src/adapters/timer/event_handlers/timer_resumed.rs` — new `TimerResumedHandler`.
- Modify `core/infra/src/adapters/timer/event_handlers/mod.rs` — module + re-export.
- Modify `core/infra/src/adapters/timer/event_handlers/registry.rs` — subscribe + unregister.
- Modify `core/infra/src/adapters/timer/event_handlers/timer_started.rs` — emit `{ task_id, state }`.
- Modify `core/infra/src/adapters/timer/event_handlers/timer_reset.rs` — emit `{ task_id, state }`.
- Modify `core/infra/src/adapters/timer/event_handlers/timer_paused.rs` — emit `{ task_id, state }`.

**Infra / audio handlers (modified + new handler in same file):**
- Modify `core/infra/src/adapters/audio/event_handlers.rs` — extract `play_phase_start_audio` helper, add `TimerResumedAudioHandler`, register it.

**Infra / notifications handlers (modified + new handler in same file):**
- Modify `core/infra/src/adapters/notifications/event_handlers.rs` — extract `send_session_started_notification` helper, add `TimerResumedNotificationHandler`, register it.

**Tests (modified + new file):**
- Create `core/infra/tests/app/timer_event_payloads.rs` — integration tests asserting the four timer UI events carry `task_id` and `state`, and that resume produces `TimerResumed` (not `TimerStarted`).
- Modify `core/infra/tests/app/mod.rs` — register the new test module.
- Modify `core/infra/tests/app/tick_loop_invariants.rs` — extend `domain_events_do_not_mutate_tick_loop` to publish a `TimerResumed` event.

**Frontend (modified):**
- Modify `apps/react-ui/src/lib/tauri.ts` — `EventPayloadMap` entries for the four events switch from `TimerStateData` to `Timer`.
- Modify `apps/react-ui/src/app/EventBus.ts` — four listeners switch from `applyTimerState` to `applyTimer`.

---

### Task 1: Add `Resumed` domain event

**Files:**
- Create: `core/domain/src/timer/events/timer_resumed.rs`
- Modify: `core/domain/src/timer/events/mod.rs:1-13` (add module) and `:15-27` (add re-export) and `:33-49` (extend test)
- Modify: `core/domain/src/lib.rs:29-37` (add re-export)
- Test: `core/domain/src/timer/events/mod.rs` (the `#[cfg(test)] mod tests` block)

**Interfaces:**
- Produces: `domain::timer::events::Resumed` struct with `pub` fields `{ task_id: TaskId, phase: Phase, duration_seconds: u32, version: u64, occurred_at: DateTime<Utc> }`, constructor `Resumed::new(task_id, phase, duration_seconds, version) -> Self`, and `Event` trait impl with `event_type() == "Resumed"`. Also re-exported as `domain::TimerResumed` and `domain::timer::Resumed` (the latter via the wildcard re-export at `core/domain/src/timer/mod.rs` — confirm by following `Started`'s path).

- [ ] **Step 1: Read `core/domain/src/timer/events/timer_started.rs` to use as the exact template.**

Run: `head -56 core/domain/src/timer/events/timer_started.rs`
Why: The new file is structurally identical to `Started`; copy its layout field-for-field, only changing the struct name and `event_type()` string.

- [ ] **Step 2: Write the failing test in `core/domain/src/timer/events/mod.rs`**

Append a new test inside the existing `mod tests` block (after `should_serialize_active_task_switched_event`, before the closing `}`):

```rust
    #[test]
    fn resumed_event_has_correct_event_type_and_serializes() {
        let resumed =
            Resumed::new(crate::TaskId::new(), Phase::Work, 1500, 1);

        assert_eq!(resumed.event_type(), "Resumed");
        assert_eq!(resumed.version(), 1);

        let serialized = serde_json::to_string(&resumed).unwrap();
        let deserialized: Resumed =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(resumed, deserialized);
    }
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test -p domain resumed_event_has_correct_event_type_and_serializes`
Expected: compile error — `Resumed` not found / not defined.

- [ ] **Step 4: Create `core/domain/src/timer/events/timer_resumed.rs`**

```rust
use crate::TaskId;
use crate::timer::Phase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resumed {
    pub task_id: TaskId,
    pub phase: Phase,
    pub duration_seconds: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl Resumed {
    pub fn new(
        task_id: TaskId,
        phase: Phase,
        duration_seconds: u32,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            phase,
            duration_seconds,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for Resumed {
    fn event_type(&self) -> &'static str {
        "Resumed"
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn clone_box(&self) -> Box<dyn crate::Event> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

- [ ] **Step 5: Register the module and re-export in `core/domain/src/timer/events/mod.rs`**

Add `pub mod timer_resumed;` to the module list (after `pub mod timer_reset;` on line 8 to keep alphabetical order). Add `pub use timer_resumed::Resumed;` to the re-export list (after `pub use timer_reset::Reset;` on line 22).

- [ ] **Step 6: Re-export as `TimerResumed` in `core/domain/src/lib.rs`**

In the `pub use timer::{ ... }` block (lines 29-37), add `Resumed as TimerResumed,` immediately after `Reset as TimerReset,` on line 32, keeping alphabetical order.

- [ ] **Step 7: Run the test to verify it passes**

Run: `cargo test -p domain resumed_event_has_correct_event_type_and_serializes`
Expected: PASS.

- [ ] **Step 8: Run the whole domain crate to confirm no regressions**

Run: `cargo test -p domain`
Expected: all tests pass.

- [ ] **Step 9: Verify clippy + fmt**

Run: `cargo clippy -p domain --all-targets -- -D warnings && cargo fmt -p domain --check`
Expected: no warnings, no diff.

- [ ] **Step 10: Commit**

```bash
git add core/domain/src/timer/events/timer_resumed.rs core/domain/src/timer/events/mod.rs core/domain/src/lib.rs
git commit -m "feat(domain): add Resumed timer domain event

Introduce a distinct domain event for timer resume, replacing the
current misuse of Started in transitions::resume(). Structurally
identical to Started; event_type() returns \"Resumed\"."
```

---

### Task 2: `transitions::resume()` emits `Resumed` instead of `Started`

**Files:**
- Modify: `core/domain/src/timer/transitions.rs:108-161` (the `resume` function) and the file's `use` block at the top.
- Test: `core/domain/src/timer/transitions.rs` (existing test module at the bottom — see lines 353+, 401+, 416+, 536+).

**Interfaces:**
- Consumes: `domain::timer::events::Resumed` from Task 1.
- Produces: `transitions::resume()` now returns a `TransitionResult` whose `events: Vec<Box<dyn Event>>` contains a single `Resumed` (was: `Started`).

- [ ] **Step 1: Find existing resume tests that assert a `Started` event**

Run: `rg -n 'resume|Started::new' core/domain/src/timer/transitions.rs`

Inspect any test in the bottom `#[cfg(test)]` block of `transitions.rs` whose name contains `resume` or `paused`. The current behavior is `transitions::resume()` building `vec![Box::new(Started::new(...))]` (lines 125-130) — any test asserting `event_type() == "Started"` for the resume path must now assert `"Resumed"`. **If no existing test exercises this assertion, add one** (Step 2). If one exists, update it (Step 2 still applies, just edit it).

- [ ] **Step 2: Write/update the failing test for resume emitting `Resumed`**

In the `#[cfg(test)]` block at the bottom of `core/domain/src/timer/transitions.rs`, add (or update the equivalent existing test to read):

```rust
    #[test]
    fn resume_emits_resumed_event_not_started() {
        let task_id = crate::TaskId::new();
        let paused = TimerState::Paused {
            paused_from: Box::new(TimerState::Working {
                remaining_seconds: 1500,
            }),
            remaining_seconds: 750,
        };

        let result = StateTransitions::resume(
            paused,
            task_id,
            &TimerConfiguration::default(),
        )
        .expect("resume must succeed from Paused");

        assert_eq!(result.events.len(), 1, "resume must emit exactly one event");
        assert_eq!(
            result.events[0].event_type(),
            "Resumed",
            "resume must emit a Resumed event, not Started"
        );
    }
```

If `StateTransitions` is not the correct path (check `transitions.rs:112` for the exact `pub fn resume` declaration — it may be a free function or method on `StateTransitions`), use the actual invocation pattern.

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test -p domain resume_emits_resumed_event_not_started`
Expected: FAIL — `event_type()` returns `"Started"`, not `"Resumed"`.

- [ ] **Step 4: Update `transitions::resume()` to emit `Resumed`**

In `core/domain/src/timer/transitions.rs`:

1. At the top of the file, locate the existing `use super::events::{...}` or `use crate::timer::events::{...}` import (the file currently imports `Started`, `Paused`, `Reset`, etc. — see line 7). Add `Resumed` to that import list (alphabetical order, after `Reset` if alphabetical, or wherever the existing list ordering places it).

2. In the body of `pub fn resume(` (around line 112), replace the event construction:

```rust
                let events: Vec<Box<dyn Event>> = vec![Box::new(Started::new(
                    task_id,
                    phase,
                    remaining_seconds,
                    1,
                ))];
```

with:

```rust
                let events: Vec<Box<dyn Event>> = vec![Box::new(Resumed::new(
                    task_id,
                    phase,
                    remaining_seconds,
                    1,
                ))];
```

(Indentation must match the existing block exactly.)

- [ ] **Step 5: Run the new test to verify it passes**

Run: `cargo test -p domain resume_emits_resumed_event_not_started`
Expected: PASS.

- [ ] **Step 6: Run the full domain test suite**

Run: `cargo test -p domain`
Expected: all tests pass. If any pre-existing test now fails because it asserted `Started` on the resume path, update it to assert `Resumed` (this is the intended behavior change).

- [ ] **Step 7: Verify clippy + fmt**

Run: `cargo clippy -p domain --all-targets -- -D warnings && cargo fmt -p domain --check`
Expected: clean.

- [ ] **Step 8: Commit**

```bash
git add core/domain/src/timer/transitions.rs
git commit -m "refactor(domain): resume emits Resumed instead of Started

The resume transition is semantically distinct from start; it now
emits a Resumed event. Audio/notification handlers will be updated
to subscribe to Resumed in subsequent tasks."
```

---

### Task 3: Add `TimerResumedHandler` (infra/timer)

**Files:**
- Create: `core/infra/src/adapters/timer/event_handlers/timer_resumed.rs`
- Modify: `core/infra/src/adapters/timer/event_handlers/mod.rs:1-22` (module + re-export)
- Modify: `core/infra/src/adapters/timer/event_handlers/registry.rs` (subscribe + unregister)
- Test: covered by Task 7's integration test (`timer_event_payloads.rs`). No standalone unit test for this handler — its behavior is identical to `TimerStartedHandler` (read state, build JSON, emit two events) and is exercised end-to-end.

**Interfaces:**
- Consumes: `domain::TimerResumed` from Task 1, `domain::event_names::ui_listeners::timer::{RESUME, STATUS_CHANGED}`, `Arc<dyn Emitter>`, `Arc<TimerTickService>`.
- Produces: `TimerResumedHandler::new(emitter, timer_srv) -> Self`, registered in the timer event handler registry. On receipt of a `TimerResumed` event, emits `timer:timer_resumed` and `timer:status_changed` with payload `{ task_id, state }`.

- [ ] **Step 1: Create the handler file `core/infra/src/adapters/timer/event_handlers/timer_resumed.rs`**

Model on `core/infra/src/adapters/timer/event_handlers/timer_started.rs` exactly. Note the payload shape change: the handler reads `event.task_id` (from the domain event) and `t.state()` (from the timer service), then builds a combined JSON object.

```rust
use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TimerResumed`.
///
/// Per the tick-loop ownership contract on
/// `TimerTickService::start_timer_tick_loop`, this handler MUST NOT drive the
/// tick loop. The orchestrator that called `resume_timer_phase` is responsible
/// for calling `start_timer_tick_loop` directly. This handler is read-only
/// with respect to `cancel_handle`.
pub struct TimerResumedHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl TimerResumedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        TimerResumedHandler { emitter, timer_srv }
    }
}

#[async_trait]
impl EventHandler for TimerResumedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerResumed>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let timer_resumed = event
            .as_any()
            .downcast_ref::<domain::TimerResumed>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to resume timer".to_string(),
            })?;

        let task_id = timer_resumed.task_id.to_string();

        // Read-only access to format the UI payload. No mutation of
        // cancel_handle. The orchestrator has already started the loop.
        let state_json =
            self.timer_srv.with_timer(|t| json!(t.state())).await;

        let payload = json!({
            "task_id": task_id,
            "state": state_json,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::RESUME,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer resumed event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                payload,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit timer status changed event: {e}"
                ),
            })?;

        Ok(())
    }
}
```

- [ ] **Step 2: Register the module and re-export in `core/infra/src/adapters/timer/event_handlers/mod.rs`**

Add `mod timer_resumed;` to the module list (after `mod timer_reset;` on line 6, alphabetical). Add `pub(super) use timer_resumed::TimerResumedHandler;` to the re-export list (after `pub(super) use timer_reset::TimerResetHandler;` on line 16).

- [ ] **Step 3: Subscribe the handler in `core/infra/src/adapters/timer/event_handlers/registry.rs`**

1. Add `TimerResumedHandler` to the `use super::{...}` import block at the top (lines 11-15, alphabetical after `TimerResetHandler`).

2. In `register_timer_handlers`, after the `TimerStartedHandler::new(...)` subscribe block (lines 47-50), add:

```rust
    event_bus.subscribe(Box::new(TimerResumedHandler::new(
        emitter.clone(),
        timer_srv.clone(),
    )))?;
```

3. In `unregister_timer_handlers`, after the `TimerStartedHandler` clear (line 72), add:

```rust
    event_bus.clear_handlers_for_type(TypeId::of::<TimerResumedHandler>())?;
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build -p infra`
Expected: compiles cleanly. If the import path for `TimerTickService` or `Emitter` differs from `timer_started.rs`, copy the exact `use` lines from that file.

- [ ] **Step 5: Verify clippy + fmt**

Run: `cargo clippy -p infra --all-targets -- -D warnings && cargo fmt -p infra --check`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add core/infra/src/adapters/timer/event_handlers/timer_resumed.rs core/infra/src/adapters/timer/event_handlers/mod.rs core/infra/src/adapters/timer/event_handlers/registry.rs
git commit -m "feat(infra): add TimerResumedHandler emitting task_id + state

Subscribes to TimerResumed and emits timer:timer_resumed and
timer:status_changed with { task_id, state } payload. Read-only
w.r.t. cancel_handle per the tick-loop ownership contract."
```

---

### Task 4: Update `TimerStartedHandler`, `TimerResetHandler`, `TimerPausedHandler` to emit `{ task_id, state }`

**Files:**
- Modify: `core/infra/src/adapters/timer/event_handlers/timer_started.rs:38-66`
- Modify: `core/infra/src/adapters/timer/event_handlers/timer_reset.rs:36-75`
- Modify: `core/infra/src/adapters/timer/event_handlers/timer_paused.rs:29-47`
- Test: covered by Task 7's integration tests.

**Interfaces:**
- Produces: the three handlers now emit `timer:timer_started`, `timer:timer_reset`, `timer:timer_paused` (and `timer:status_changed` for started/reset) with payload `{ task_id, state }` instead of bare `state`.

- [ ] **Step 1: Update `TimerStartedHandler` in `core/infra/src/adapters/timer/event_handlers/timer_started.rs`**

Rename the unused binding `let _timer_started = ...` to `let timer_started = ...` (it's now needed for `task_id`). Replace the body from the `let state_json = ...` line through both `.emit(...)` calls with:

```rust
        let task_id = timer_started.task_id.to_string();

        // Read-only access to format the UI payload. No mutation of
        // cancel_handle. The orchestrator has already started the loop.
        let state_json =
            self.timer_srv.with_timer(|t| json!(t.state())).await;

        let payload = json!({
            "task_id": task_id,
            "state": state_json,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::START,
                payload,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;

        Ok(())
```

Note: both `START` and `STATUS_CHANGED` previously received `state_json` / `state_json.clone()`. Replace those with `payload` / `payload.clone()`.

- [ ] **Step 2: Update `TimerResetHandler` in `core/infra/src/adapters/timer/event_handlers/timer_reset.rs`**

Rename `_timer_reset` to `timer_reset`. Replace the body from the `let state_json = ...` line through both `.emit(...)` calls with:

```rust
        let task_id = timer_reset.task_id.to_string();

        // Read-only: format the current timer state for the UI. The
        // orchestrator has already stopped the loop and refreshed state.
        let state_json = self
            .timer_srv
            .with_timer(|t| {
                log::info!("{:?} timer reset", t);
                json!(t.state())
            })
            .await;

        let payload = json!({
            "task_id": task_id,
            "state": state_json,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::RESET,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer reset event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                payload,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit timer status changed event: {e}"
                ),
            })?;

        Ok(())
```

- [ ] **Step 3: Update `TimerPausedHandler` in `core/infra/src/adapters/timer/event_handlers/timer_paused.rs`**

Rename `_timer_paused` to `timer_paused`. Replace the body from the `let state_json = ...` line through the `.emit(...)` call with:

```rust
        let task_id = timer_paused.task_id.to_string();

        // The orchestrator that called `pause_timer_phase` is responsible for
        // stop_timer_tick_loop + load_state. This handler is a UI-only emitter.
        let state_json =
            self.timer_srv.with_timer(|t| json!(t.state())).await;

        let payload = json!({
            "task_id": task_id,
            "state": state_json,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::PAUSE,
                payload,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer paused event: {e}"),
            })?;
        Ok(())
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build -p infra`
Expected: compiles cleanly.

- [ ] **Step 5: Verify clippy + fmt**

Run: `cargo clippy -p infra --all-targets -- -D warnings && cargo fmt -p infra --check`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add core/infra/src/adapters/timer/event_handlers/timer_started.rs core/infra/src/adapters/timer/event_handlers/timer_reset.rs core/infra/src/adapters/timer/event_handlers/timer_paused.rs
git commit -m "refactor(infra): timer UI handlers emit { task_id, state } payload

TimerStarted/Reset/Paused handlers previously emitted bare state and
discarded the task_id carried by the domain event. The frontend can
now populate timer.task_id directly from the payload."
```

---

### Task 5: Audio — extract `play_phase_start_audio` helper + add `TimerResumedAudioHandler`

**Files:**
- Modify: `core/infra/src/adapters/audio/event_handlers.rs` (lines 1-9 imports, 248-302 `TimerStartedAudioHandler`, 391-433 registry function).
- Test: existing audio tests pass — the helper is exercised by `TimerStartedAudioHandler`'s existing tests; `TimerResumedAudioHandler` reuses the same logic.

**Interfaces:**
- Produces: a free async function `play_phase_start_audio(audio_service: &AudioServiceWrapper, config_repository: &Arc<dyn ConfigRepository + Send + Sync>, phase: Phase) -> Result<()>`; a new `TimerResumedAudioHandler` struct subscribing to `domain::timer::events::Resumed`.

- [ ] **Step 1: Add `Resumed` to the imports at the top of `core/infra/src/adapters/audio/event_handlers.rs`**

Modify the existing import block (lines 2-6):

```rust
use domain::timer::events::{
    BreakPhaseCompleted, BreakPhaseStarted, Paused as TimerPaused,
    Resumed as TimerResumed, Started as TimerStarted, Tick as TimerTick,
    WorkPhaseCompleted, WorkPhaseStarted,
};
```

- [ ] **Step 2: Extract the body of `TimerStartedAudioHandler::handle` into a free function**

Place this free function immediately **above** the `pub struct TimerStartedAudioHandler {` line (currently around line 248):

```rust
/// Shared body of `TimerStartedAudioHandler` and `TimerResumedAudioHandler`.
/// Plays the phase-start sound asset configured for `phase` (work vs. break)
/// unless audio is muted.
async fn play_phase_start_audio(
    audio_service: &AudioServiceWrapper,
    config_repository: &Arc<dyn ConfigRepository + Send + Sync>,
    phase: Phase,
) -> Result<()> {
    let config = config_repository.get_config().await?;

    if config.audio.muted {
        return Ok(());
    }

    let asset_id = match phase {
        Phase::Work => config
            .audio
            .work_notification_sound
            .unwrap_or_else(|| "bell".to_string()),
        Phase::ShortBreak | Phase::LongBreak => config
            .audio
            .break_notification_sound
            .unwrap_or_else(|| "gentle-bell".to_string()),
    };

    let request = PlaybackRequest::new(asset_id, config.audio.volume)?;

    audio_service.play_audio(request)?;
    Ok(())
}
```

- [ ] **Step 3: Replace the body of `TimerStartedAudioHandler::handle` with a call to the helper**

The handler now reads:

```rust
    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(timer_started) =
            event.as_any().downcast_ref::<TimerStarted>()
        {
            play_phase_start_audio(
                &self.audio_service,
                &self.config_repository,
                timer_started.phase,
            )
            .await?;
        }
        Ok(())
    }
```

- [ ] **Step 4: Add `TimerResumedAudioHandler` immediately below `TimerStartedAudioHandler`**

Place it directly after `TimerStartedAudioHandler`'s `impl` block (before `pub struct TimerPausedAudioHandler`):

```rust
pub struct TimerResumedAudioHandler {
    audio_service: Arc<AudioServiceWrapper>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl TimerResumedAudioHandler {
    pub fn new(
        audio_service: Arc<AudioServiceWrapper>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        Self {
            audio_service,
            config_repository,
        }
    }
}

#[async_trait]
impl EventHandler for TimerResumedAudioHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerResumed>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(timer_resumed) =
            event.as_any().downcast_ref::<TimerResumed>()
        {
            play_phase_start_audio(
                &self.audio_service,
                &self.config_repository,
                timer_resumed.phase,
            )
            .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerResumedAudioHandler"
    }
}
```

- [ ] **Step 5: Subscribe `TimerResumedAudioHandler` in `register_audio_event_handlers`**

In the registry function (around lines 417-420, immediately after the `TimerStartedAudioHandler::new(...)` subscribe block), add:

```rust
    let _ = event_bus.subscribe(Box::new(TimerResumedAudioHandler::new(
        audio_service.clone(),
        config_repository.clone(),
    )));
```

- [ ] **Step 6: Verify compilation**

Run: `cargo build -p infra`
Expected: compiles cleanly.

- [ ] **Step 7: Verify clippy + fmt**

Run: `cargo clippy -p infra --all-targets -- -D warnings && cargo fmt -p infra --check`
Expected: clean.

- [ ] **Step 8: Run audio tests**

Run: `cargo test -p infra audio`
Expected: all pass. The behavior of `TimerStartedAudioHandler` is unchanged (same helper, same inputs); the new handler is exercised by Task 7's integration test for the resume path.

- [ ] **Step 9: Commit**

```bash
git add core/infra/src/adapters/audio/event_handlers.rs
git commit -m "feat(infra/audio): add TimerResumedAudioHandler

Extract the phase-start audio logic from TimerStartedAudioHandler
into a shared play_phase_start_audio helper, and add a parallel
TimerResumedAudioHandler that subscribes to TimerResumed so resume
continues to play a sound after the domain event split."
```

---

### Task 6: Notifications — extract `send_session_started_notification` helper + add `TimerResumedNotificationHandler`

**Files:**
- Modify: `core/infra/src/adapters/notifications/event_handlers.rs` (lines 1-12 imports, 14-63 `TimerStartedNotificationHandler`, 272-307 registry function).

**Interfaces:**
- Produces: a free async function `send_session_started_notification(notification_service: &Arc<dyn NotificationServiceTrait>, task_repository: &Arc<dyn domain::TaskRepository + Send + Sync>, task_id: domain::TaskId) -> Result<()>`; a new `TimerResumedNotificationHandler` subscribing to `domain::timer::events::Resumed`.

- [ ] **Step 1: Add `Resumed` to the imports at the top of `core/infra/src/adapters/notifications/event_handlers.rs`**

Modify the existing import block (lines 2-5):

```rust
use domain::timer::events::{
    BreakPhaseCompleted, BreakPhaseStarted, Paused as TimerPaused,
    Resumed as TimerResumed, Started as TimerStarted, WorkPhaseCompleted,
};
```

Also add `TaskId` to the existing `use domain::{...}` import (line 6) if not already present:

```rust
use domain::{Event, Result, TaskCompleted, TaskId};
```

- [ ] **Step 2: Extract the body of `TimerStartedNotificationHandler::handle` into a free function**

Place this free function immediately **above** the `pub struct TimerStartedNotificationHandler {` line (currently line 14):

```rust
/// Shared body of `TimerStartedNotificationHandler` and
/// `TimerResumedNotificationHandler`. Looks up the task name and posts a
/// `SessionStarted` OS notification for the given phase.
async fn send_session_started_notification(
    notification_service: &Arc<dyn NotificationServiceTrait>,
    task_repository: &Arc<dyn domain::TaskRepository + Send + Sync>,
    task_id: TaskId,
    phase: Phase,
) -> Result<()> {
    let task_name = task_repository
        .get_all()
        .await?
        .into_iter()
        .find(|t| t.id() == task_id)
        .map(|task| task.name().to_string());

    let notification_event = NotificationEvent::SessionStarted {
        phase,
        task_name,
    };
    notification_service
        .send_notification(notification_event)
        .await?;
    Ok(())
}
```

If `NotificationEvent` and `Phase` are not currently in scope at the top of the file, add them to the existing imports. `NotificationEvent` comes from `super::service::{NotificationEvent, NotificationServiceTrait}` (line 10). `Phase` comes from `domain::Phase` — add it to the `use domain::{...}` line.

- [ ] **Step 3: Replace the body of `TimerStartedNotificationHandler::handle` with a call to the helper**

```rust
    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(timer_started) =
            event.as_any().downcast_ref::<TimerStarted>()
        {
            send_session_started_notification(
                &self.notification_service,
                &self.task_repository,
                timer_started.task_id,
                timer_started.phase,
            )
            .await?;
        }
        Ok(())
    }
```

- [ ] **Step 4: Add `TimerResumedNotificationHandler` immediately below `TimerStartedNotificationHandler`**

Place it directly after `TimerStartedNotificationHandler`'s `impl` block (before `pub struct TimerPausedNotificationHandler`):

```rust
pub struct TimerResumedNotificationHandler {
    notification_service: Arc<dyn NotificationServiceTrait>,
    task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
}

impl TimerResumedNotificationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationServiceTrait>,
        task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    ) -> Self {
        Self {
            notification_service,
            task_repository,
        }
    }
}

#[async_trait]
impl EventHandler for TimerResumedNotificationHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerResumed>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(timer_resumed) =
            event.as_any().downcast_ref::<TimerResumed>()
        {
            send_session_started_notification(
                &self.notification_service,
                &self.task_repository,
                timer_resumed.task_id,
                timer_resumed.phase,
            )
            .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerResumedNotificationHandler"
    }
}
```

- [ ] **Step 5: Subscribe `TimerResumedNotificationHandler` in `register_notification_handlers`**

In the registry function (around lines 277-281, immediately after the `TimerStartedNotificationHandler::new(...)` subscribe block), add:

```rust
    let _ = event_bus.subscribe(Box::new(
        TimerResumedNotificationHandler::new(
            notification_service.clone(),
            task_repository.clone(),
        ),
    ));
```

- [ ] **Step 6: Verify compilation**

Run: `cargo build -p infra`
Expected: compiles cleanly.

- [ ] **Step 7: Verify clippy + fmt**

Run: `cargo clippy -p infra --all-targets -- -D warnings && cargo fmt -p infra --check`
Expected: clean.

- [ ] **Step 8: Run notification tests**

Run: `cargo test -p infra notification`
Expected: all pass.

- [ ] **Step 9: Commit**

```bash
git add core/infra/src/adapters/notifications/event_handlers.rs
git commit -m "feat(infra/notifications): add TimerResumedNotificationHandler

Extract the session-started notification logic from
TimerStartedNotificationHandler into a shared
send_session_started_notification helper, and add a parallel
TimerResumedNotificationHandler so resume continues to post an OS
notification after the domain event split."
```

---

### Task 7: Integration tests for the four timer UI event payloads + resume domain event

**Files:**
- Create: `core/infra/tests/app/timer_event_payloads.rs`
- Modify: `core/infra/tests/app/mod.rs` (add `mod timer_event_payloads;`)
- Modify: `core/infra/tests/app/tick_loop_invariants.rs:19-23` (comment) and `:77-102` (publish a `TimerResumed` event too).

**Interfaces:**
- Consumes: `domain::TimerResumed` (Task 1), `usecases::timer::{start_timer_phase, pause_timer_phase, resume_timer_phase, reset_timer_phase}` and the `StartTimerPhaseCmd` struct, plus the test-context helpers (`AppContextBuilder`, `TaskBuilder`, `utils::setup::setup_ctx`).

- [ ] **Step 1: Register the new test module in `core/infra/tests/app/mod.rs`**

Open the file and add `mod timer_event_payloads;` to the module list (alphabetical, after `mod timer;`).

- [ ] **Step 2: Create the failing tests in `core/infra/tests/app/timer_event_payloads.rs`**

```rust
use std::time::Duration;

use domain::{
    Config, EventPublisher, Phase, TaskRepository, TaskStatus,
    TaskCyclingBehavior, event_names,
};
use usecases::timer::{
    StartTimerPhaseCmd, pause_timer_phase, reset_timer_phase,
    resume_timer_phase, start_timer_phase,
};

use crate::{AppContextBuilder, TaskBuilder, utils::setup::setup_ctx};

/// Helper: start a fresh timer bound to a task. Returns the task id.
async fn start_timer_for_task(ctx: &crate::AppContext) -> domain::TaskId {
    let task = TaskBuilder::new()
        .name("Payload test")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    let task_id = task.id();
    ctx.task_repo.create(task).await.unwrap();

    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd {
            task_id: Some(task_id),
        },
    )
    .await
    .expect("start_timer_phase failed");

    tokio::time::sleep(Duration::from_millis(150)).await;
    task_id
}

#[tokio::test]
async fn timer_started_payload_carries_task_id_and_state() {
    let ctx = setup_ctx("timer_started_payload_carries_task_id_and_state")
        .await;
    let task_id = start_timer_for_task(&ctx).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::START);
    assert!(
        !events.is_empty(),
        "timer:timer_started was not emitted"
    );

    let payload = &events[0].payload;
    let embedded_task_id =
        payload.get("task_id").expect("payload missing task_id");
    assert_eq!(
        *embedded_task_id,
        serde_json::json!(task_id.to_string()),
        "payload task_id must match the started task"
    );
    assert!(
        payload.get("state").is_some(),
        "payload missing state field"
    );
}

#[tokio::test]
async fn timer_paused_payload_carries_task_id_and_state() {
    let ctx = setup_ctx("timer_paused_payload_carries_task_id_and_state")
        .await;
    let task_id = start_timer_for_task(&ctx).await;

    pause_timer_phase(
        task_id,
        750,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("pause_timer_phase failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::PAUSE);
    assert!(
        !events.is_empty(),
        "timer:timer_paused was not emitted"
    );

    let payload = &events[0].payload;
    assert_eq!(
        *payload.get("task_id").expect("payload missing task_id"),
        serde_json::json!(task_id.to_string())
    );
    assert!(payload.get("state").is_some());
}

#[tokio::test]
async fn timer_resumed_payload_carries_task_id_and_state() {
    let ctx = setup_ctx("timer_resumed_payload_carries_task_id_and_state")
        .await;
    let task_id = start_timer_for_task(&ctx).await;

    // Pause, then resume — resume must emit timer:timer_resumed (not
    // timer:timer_started) with { task_id, state }.
    pause_timer_phase(
        task_id,
        750,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("pause_timer_phase failed");
    tokio::time::sleep(Duration::from_millis(150)).await;

    ctx.ui_simulator.app_handle().clear_events();

    resume_timer_phase(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("resume_timer_phase failed");
    tokio::time::sleep(Duration::from_millis(150)).await;

    let resume_events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::RESUME);
    assert!(
        !resume_events.is_empty(),
        "timer:timer_resumed was not emitted"
    );

    let payload = &resume_events[0].payload;
    assert_eq!(
        *payload.get("task_id").expect("payload missing task_id"),
        serde_json::json!(task_id.to_string())
    );
    assert!(payload.get("state").is_some());

    // Negative assertion: resume must NOT have emitted timer:timer_started
    // (it now uses a distinct Resumed domain event).
    let started_events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::START);
    assert!(
        started_events.is_empty(),
        "resume must not emit timer:timer_started; expected timer:timer_resumed only"
    );
}

#[tokio::test]
async fn timer_reset_payload_carries_task_id_and_state() {
    let ctx = setup_ctx("timer_reset_payload_carries_task_id_and_state")
        .await;
    let task_id = start_timer_for_task(&ctx).await;

    reset_timer_phase(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("reset_timer_phase failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::RESET);
    assert!(
        !events.is_empty(),
        "timer:timer_reset was not emitted"
    );

    let payload = &events[0].payload;
    assert_eq!(
        *payload.get("task_id").expect("payload missing task_id"),
        serde_json::json!(task_id.to_string())
    );
    assert!(payload.get("state").is_some());
}
```

If the exact signature of `reset_timer_phase` differs from `(task_id, task_repo, timer_repo, event_bus)` (check `core/usecases/src/timer/reset_timer_phase.rs`), adjust the call to match.

- [ ] **Step 3: Run the tests to verify they fail or pass appropriately**

Run: `cargo test -p infra --test app timer_event_payloads`

Expected behavior depends on task completion state:
- If Tasks 1-6 are done: all four tests PASS.
- If run before Tasks 1-6: the resume test fails (no `TimerResumed` event, no `timer:timer_resumed` emission); the started/paused/reset tests fail because the payload lacks `task_id`.

Since Tasks 1-6 are now complete, expect PASS. If any fails, investigate (likely a signature mismatch — fix the call site to match the actual usecase signature).

- [ ] **Step 4: Extend the tick-loop invariant test in `core/infra/tests/app/tick_loop_invariants.rs`**

In `domain_events_do_not_mutate_tick_loop`:

1. Update the doc comment block (lines 19-23) to mention `TimerResumed`.
2. After the `started_event` construction (lines 90-95) and before the `task_reset_event` construction (lines 96-97), add:

```rust
    let resumed_event = Box::new(domain::TimerResumed::new(
        task_id,
        phase,
        timer_config.work_duration.as_secs() as u32,
        1,
    ));
```

3. After `ctx.event_bus.publish(started_event);` (line 101), add:

```rust
    ctx.event_bus.publish(resumed_event);
```

- [ ] **Step 5: Run the invariant test to verify it passes**

Run: `cargo test -p infra --test app domain_events_do_not_mutate_tick_loop`
Expected: PASS — `TimerResumedHandler` is read-only w.r.t. `cancel_handle`.

- [ ] **Step 6: Run the full infra test suite**

Run: `cargo test -p infra`
Expected: all tests pass. If any pre-existing resume-related test breaks because it asserted `TimerStarted` was published/subscribed on resume, update it to reference `TimerResumed`.

- [ ] **Step 7: Verify clippy + fmt**

Run: `cargo clippy -p infra --all-targets -- -D warnings && cargo fmt -p infra --check`
Expected: clean.

- [ ] **Step 8: Commit**

```bash
git add core/infra/tests/app/timer_event_payloads.rs core/infra/tests/app/mod.rs core/infra/tests/app/tick_loop_invariants.rs
git commit -m "test(infra): assert timer UI events carry task_id + state

Adds timer_event_payloads.rs covering timer:timer_started/paused/
resumed/reset payloads, including a negative assertion that resume
no longer emits timer:timer_started. Extends the tick-loop invariant
test to publish a TimerResumed event."
```

---

### Task 8: Frontend — switch the four timer listeners to `applyTimer`

**Files:**
- Modify: `apps/react-ui/src/lib/tauri.ts:170-187` (the `EventPayloadMap` interface)
- Modify: `apps/react-ui/src/app/EventBus.ts:53-56` (the four `onEvent` calls)

**Interfaces:**
- Consumes: the `Timer` interface from `apps/react-ui/src/pages/timer/useTimer.ts:33-36` (already imported into `tauri.ts`).
- Produces: `EventPayloadMap['timer:timer_reset' | 'timer:timer_started' | 'timer:timer_paused' | 'timer:timer_resumed']` is now `Timer`. `EventBus.ts` routes these to `applyTimer` instead of `applyTimerState`.

- [ ] **Step 1: Update `EventPayloadMap` in `apps/react-ui/src/lib/tauri.ts`**

Replace lines 181-184 (the four `TimerStateData` entries):

```ts
  'timer:timer_reset': TimerStateData,
  'timer:timer_started': TimerStateData,
  'timer:timer_paused': TimerStateData,
  'timer:timer_resumed': TimerStateData,
```

with:

```ts
  'timer:timer_reset': Timer,
  'timer:timer_started': Timer,
  'timer:timer_paused': Timer,
  'timer:timer_resumed': Timer,
```

If `TimerStateData` becomes unused after this change (check the rest of the file — it is still referenced by the `Timer` interface itself at line 30, so the type stays imported; just remove any now-orphan usage if introduced by an editor auto-fix), leave the import alone.

- [ ] **Step 2: Update `EventBus.ts` to use `applyTimer` for the four events**

In `apps/react-ui/src/app/EventBus.ts:53-56`, replace:

```ts
      onEvent(events.timerReset, applyTimerState),
      onEvent(events.timerPaused, applyTimerState),
      onEvent(events.timerStarted, applyTimerState),
      onEvent(events.timerResumed, applyTimerState),
```

with:

```ts
      onEvent(events.timerReset, applyTimer),
      onEvent(events.timerPaused, applyTimer),
      onEvent(events.timerStarted, applyTimer),
      onEvent(events.timerResumed, applyTimer),
```

`applyTimer` is already selected from the store at line 36. `applyTimerState` stays imported because it is still used by `timerPhaseCompleted` at line 47.

- [ ] **Step 3: Verify typecheck**

Run: `pnpm --filter react-ui typecheck`
Expected: PASS. TypeScript confirms the new payload shape (`Timer`) is assignable to `applyTimer`'s parameter.

- [ ] **Step 4: Verify lint**

Run: `pnpm --filter react-ui lint`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/react-ui/src/lib/tauri.ts apps/react-ui/src/app/EventBus.ts
git commit -m "refactor(react-ui): timer events route to applyTimer with task_id

The four timer UI events now carry { task_id, state } payloads; the
EventBus applies them with applyTimer (replaces the whole timer)
instead of applyTimerState (which preserved the stale task_id)."
```

---

## Final Verification

After all 8 tasks are merged:

- [ ] **Full Rust verification**

```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

Expected: all green.

- [ ] **Frontend verification**

```bash
pnpm --filter react-ui typecheck
pnpm --filter react-ui lint
pnpm --filter react-ui build
```

Expected: all green.

- [ ] **Manual smoke test**

1. Launch the app, create a task, start a work session.
2. In DevTools (Zustand store inspector or a `console.log` in `applyTimer`), confirm `timer.task_id` is set from the event payload.
3. Pause the timer — `timer.task_id` persists via the pause event payload (not via preservation).
4. Resume — confirm `timer:timer_resumed` arrives and `timer.task_id` is still correct; confirm no `timer:timer_started` is emitted on resume.
5. Reset — `timer.task_id` follows the reset event payload.
6. (Audio/notification) Resume should still play the configured sound and post a notification.

## Self-Review

**Spec coverage check:**

- Spec §"Payload shape" → Task 3 (resumed), Task 4 (started/paused/reset), Task 7 (integration tests). ✓
- Spec §"New event: `Resumed`" → Task 1. ✓
- Spec §"`transitions::resume()` emits `Resumed`" → Task 2. ✓
- Spec §"`TimerResumedHandler` (new)" → Task 3. ✓
- Spec §"Update `TimerStartedHandler`, `TimerResetHandler`, `TimerPausedHandler`" → Task 4. ✓
- Spec §"Infra layer — audio handlers" → Task 5. ✓
- Spec §"Infra layer — notification handlers" → Task 6. ✓
- Spec §"Frontend — `tauri.ts`" → Task 8 step 1. ✓
- Spec §"Frontend — `EventBus.ts`" → Task 8 step 2. ✓
- Spec §"Domain unit tests" → Task 1 step 2. ✓
- Spec §"Infra integration tests" → Task 7. ✓
- Spec §"Lock the new `TimerResumedHandler` under the same tick-loop ownership invariant" → Task 7 step 4. ✓

**Placeholder scan:** no TBD / "implement later" / "similar to". Each step contains the actual code or commands.

**Type consistency check:** `TimerResumedHandler::new(emitter, timer_srv)` consistent across Task 3 (definition) and Task 3 step 3 (registration). `play_phase_start_audio` signature consistent across Task 5 step 2 (definition) and step 3/4 (callers). `send_session_started_notification` signature consistent across Task 6 step 2 (definition) and step 3/4 (callers). Frontend `Timer` interface usage consistent across Task 8 steps 1-2. ✓
