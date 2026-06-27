# Task Switch Resets Timer to Idle — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make switching the active task always leave the timer bound to the new task in the `Idle` state, abandoning the old task's in-progress pomodoro — fixing the regression where the timer's phase/countdown carried over to the new task.

**Architecture:** The fix lives in the usecase layer. `switch_active_task` and `switch_task` currently rebind `task_id` while preserving the old `TimerState` (via `Timer::with_state(.., old_state...)`). They will instead use the existing `Timer::new(task_id)` constructor, which already encodes the invariant "bound task starts in `Idle`" (`core/domain/src/timer/timer.rs:118-125`). A same-task idempotency guard prevents a no-op switch from stopping a running timer. No orchestrator, tick-loop, or domain-event changes are required — the existing orchestrator only restarts the tick loop when `is_running()`, which is false after the switch.

**Tech Stack:** Rust, Tauri 2, async-trait, existing `InMemoryEventBus` (unchanged). Tests are integration tests under `core/infra/tests/app/adv_timer.rs`.

## Global Constraints

- **Dependency rule:** dependencies point inward toward `domain`. No new reference to `infra::adapters::TimerTickService` from `core/usecases/**` or `core/domain/**`. (This plan touches neither.)
- **No new deps:** do not add Cargo dependencies.
- **Do NOT emit a timer domain event (`TimerReset`) from the usecases.** `TimerResetHandler` reads the tick-service's stale in-memory cache before the orchestrator's `load_state()` runs; a usecase-published event would broadcast the old task's state. See spec Decision 2.
- **Do NOT clear `Timer.task_id`.** The bound `task_id` *is* the active task; clearing it yields no active task. Use `Timer::new(new_task_id)` (Active + Idle). See spec Decision 1.
- **Style:** match existing conventions. Run `cargo fmt` before each commit.
- **Each task ends with `cargo test` green for the workspace and a commit.**
- **Tick-loop ownership contract (CLAUDE.md):** only orchestrators drive `start/stop_timer_tick_loop` / `load_state`. This plan does not touch that contract.

## File Structure

| File | Responsibility after this plan |
|------|--------------------------------|
| `core/usecases/src/task/switch_active_task.rs` | Rebinds new task with a fresh `Idle` state via `Timer::new`; adds same-task idempotency guard. Deletes the carry-state `with_remaining_seconds` logic. |
| `core/usecases/src/task/switch_task.rs` | Aligned: rebinds new task with `Timer::new` (Idle). Deletes the "preserve state" branch. Keeps the strict running-guard. |
| `core/infra/tests/app/adv_timer.rs` | Updates the existing Working-path switch test to expect `Idle`; adds ShortBreak/LongBreak/Paused/same-task regression tests for `switch_active_task`; adds a Paused regression test for `switch_task`. |

**Unchanged on purpose:** `core/domain/**`, `core/infra/**` (handlers/services), `apps/tauri-app/src/commands/timer_cmd/switch_active_task.rs` (orchestrator already correct), the frontend.

---

## Task 1: `switch_active_task` resets the new task to Idle (the core fix)

TDD: write/update all the `switch_active_task` tests first, watch them fail against current carry-state behavior, then implement the one-line fix.

**Files:**
- Modify: `core/infra/tests/app/adv_timer.rs` (update existing test at line 825 + add 3 regression tests)
- Modify: `core/usecases/src/task/switch_active_task.rs:55-71`

**Interfaces:**
- Consumes: `domain::Timer::new(task_id) -> Timer` (existing, `core/domain/src/timer/timer.rs:120`).
- Produces: `switch_active_task` leaving the singleton timer as `Active { task_id: new, state: Idle }`.

- [ ] **Step 1: Update the existing Working-path test to expect Idle**

In `core/infra/tests/app/adv_timer.rs`, the test `should_switch_active_task_during_timer_session` (starts at line 825) currently asserts the timer stays `Running` with `25 * 60` seconds after the switch, then ticks task2 to completion. Under the new contract the timer is `Idle` after the switch, so it must be (re)started before the tick-to-completion loop.

Replace this block (currently lines 898-906):

```rust
    // Get state after switch
    let state_after_switch = get_timer(&ctx).await.state().clone();
    let active_task_after_switch = get_timer(&ctx).await.task_id();
    let remaining_after_switch = state_after_switch.remaining_seconds();
    let status_after_switch = state_after_switch.status();

    // Switching tasks resets the new task's timer to its full work duration
    // (25 minutes = 1500 ticks).
    let mut ticks_completed = 0;
```

with:

```rust
    // Get state after switch
    let state_after_switch = get_timer(&ctx).await.state().clone();
    let active_task_after_switch = get_timer(&ctx).await.task_id();
    let remaining_after_switch = state_after_switch.remaining_seconds();
    let status_after_switch = state_after_switch.status();

    // After switching, the timer is Idle and bound to task2. Start a fresh
    // work phase for task2 so the session-credit check below is meaningful.
    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd {
            task_id: Some(task2.id()),
        },
    )
    .await
    .expect("Failed to start task2 after switch");

    // task2's work phase is 25 minutes = 1500 ticks.
    let mut ticks_completed = 0;
```

Then replace the assertion block (currently lines 972-981):

```rust
    assert_eq!(
        remaining_after_switch,
        25 * 60,
        "Timer should reset to the new task's full work duration"
    );
    assert_eq!(
        status_after_switch,
        TimerStatus::Running,
        "Timer should continue running"
    );
```

with:

```rust
    assert_eq!(
        remaining_after_switch,
        0,
        "Timer should be Idle (0 remaining) after switch"
    );
    assert_eq!(
        status_after_switch,
        TimerStatus::Stopped,
        "Timer should be stopped after switch"
    );
```

(`TimerState::Idle` maps to `Status::Stopped` and `remaining_seconds() == 0` — see `core/domain/src/timer/state_machine.rs:166-180` and `:222-230`.)

- [ ] **Step 2: Add three regression tests for the break/Paused phases**

Append these three tests immediately after the closing brace of `should_switch_active_task_during_timer_session` (currently line 994). They construct the starting timer state directly via `domain::Timer::with_state` + `ctx.timer_repo.save`, which is the most reliable way to set the exact precondition "timer is in phase X bound to task1."

```rust
// Test 29: Switching during a ShortBreak resets the new task to Idle —
// the new task must NOT inherit the previous task's (unearned) break.
#[tokio::test]
async fn should_switch_active_task_from_short_break_resets_to_idle() {
    let ctx = setup_ctx("should_switch_active_task_from_short_break_resets_to_idle")
        .await;

    // Arrange: task1 is active and its timer is mid-ShortBreak.
    let task1 = TaskBuilder::new()
        .name("Task on break")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .build();
    ctx.task_repo
        .create(task1.clone())
        .await
        .expect("Failed to create task1");

    let task2 = TaskBuilder::new()
        .name("Incoming task")
        .max_sessions(4)
        .status(TaskStatus::Queued)
        .build();
    ctx.task_repo
        .create(task2.clone())
        .await
        .expect("Failed to create task2");

    let break_timer = domain::Timer::with_state(
        task1.id(),
        TimerState::ShortBreak { remaining_seconds: 300 },
    );
    ctx.timer_repo
        .save(&break_timer)
        .await
        .expect("Failed to save break timer");

    // Act: switch to task2.
    let switch_result = usecases::task::switch_active_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        usecases::task::SwitchActiveTaskCmd {
            task_id: task2.id(),
            old_task_id: Some(task1.id()),
        },
    )
    .await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Assert: new task is active, timer is Idle (no carried-over break).
    let timer_after = get_timer(&ctx).await;
    assert!(switch_result.is_ok(), "Failed to switch task");
    assert_eq!(
        timer_after.task_id(),
        Some(task2.id()),
        "Active task should be task2 after switch"
    );
    assert!(
        timer_after.state().is_idle(),
        "Timer should be Idle after switch, not a carried-over ShortBreak"
    );
    assert_eq!(timer_after.state().status(), TimerStatus::Stopped);
    assert_eq!(timer_after.state().remaining_seconds(), 0);
}

// Test 30: Switching during a LongBreak resets the new task to Idle.
#[tokio::test]
async fn should_switch_active_task_from_long_break_resets_to_idle() {
    let ctx = setup_ctx("should_switch_active_task_from_long_break_resets_to_idle")
        .await;

    let task1 = TaskBuilder::new()
        .name("Task on long break")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .build();
    ctx.task_repo
        .create(task1.clone())
        .await
        .expect("Failed to create task1");

    let task2 = TaskBuilder::new()
        .name("Incoming task")
        .max_sessions(4)
        .status(TaskStatus::Queued)
        .build();
    ctx.task_repo
        .create(task2.clone())
        .await
        .expect("Failed to create task2");

    let break_timer = domain::Timer::with_state(
        task1.id(),
        TimerState::LongBreak { remaining_seconds: 900 },
    );
    ctx.timer_repo
        .save(&break_timer)
        .await
        .expect("Failed to save long-break timer");

    let switch_result = usecases::task::switch_active_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        usecases::task::SwitchActiveTaskCmd {
            task_id: task2.id(),
            old_task_id: Some(task1.id()),
        },
    )
    .await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let timer_after = get_timer(&ctx).await;
    assert!(switch_result.is_ok(), "Failed to switch task");
    assert_eq!(timer_after.task_id(), Some(task2.id()));
    assert!(
        timer_after.state().is_idle(),
        "Timer should be Idle after switch, not a carried-over LongBreak"
    );
}

// Test 31: Switching while Paused resets the new task to Idle — the paused
// phase is discarded, not transferred.
#[tokio::test]
async fn should_switch_active_task_from_paused_resets_to_idle() {
    let ctx =
        setup_ctx("should_switch_active_task_from_paused_resets_to_idle").await;

    let task1 = TaskBuilder::new()
        .name("Paused task")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .build();
    ctx.task_repo
        .create(task1.clone())
        .await
        .expect("Failed to create task1");

    let task2 = TaskBuilder::new()
        .name("Incoming task")
        .max_sessions(4)
        .status(TaskStatus::Queued)
        .build();
    ctx.task_repo
        .create(task2.clone())
        .await
        .expect("Failed to create task2");

    let paused_timer = domain::Timer::with_state(
        task1.id(),
        TimerState::Paused {
            paused_from: Box::new(TimerState::Working {
                remaining_seconds: 600,
            }),
            remaining_seconds: 600,
        },
    );
    ctx.timer_repo
        .save(&paused_timer)
        .await
        .expect("Failed to save paused timer");

    let switch_result = usecases::task::switch_active_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        usecases::task::SwitchActiveTaskCmd {
            task_id: task2.id(),
            old_task_id: Some(task1.id()),
        },
    )
    .await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let timer_after = get_timer(&ctx).await;
    assert!(switch_result.is_ok(), "Failed to switch task");
    assert_eq!(timer_after.task_id(), Some(task2.id()));
    assert!(
        timer_after.state().is_idle(),
        "Timer should be Idle after switch, not a carried-over Paused state"
    );
}
```

- [ ] **Step 3: Run the tests and confirm they FAIL**

Run: `cargo test -p infra --test main -- should_switch_active_task`
Expected: FAIL. The Working-path test fails on `status == Stopped` (current behavior is `Running`); the ShortBreak/LongBreak/Paused tests fail on `is_idle()` (current behavior carries the old phase forward).

> If the Working-path test passes, the assertion edits in Step 1 were not applied. If a break/Paused test passes, the starting timer state was not actually saved — re-check the `ctx.timer_repo.save` call.

- [ ] **Step 4: Implement the reset-to-Idle fix**

In `core/usecases/src/task/switch_active_task.rs`, replace lines 55-71 (the comment, the `new_timer_remaining` computation, the interleaved `task_repo.update(task)`, the carry-state comment, and the `with_state(..., with_remaining_seconds(...))` block):

```rust
    // The new task starts a fresh work phase at its full work duration — the
    // previous task's in-progress countdown does not carry over.
    let new_timer_remaining =
        task.config().timer.work_duration.as_secs() as u32;

    task_repo.update(task).await?;

    // Create a new timer for the new task. The phase/status carry over from the
    // previous task's timer, but the remaining countdown is reset to the new
    // task's full work duration.
    let new_timer = domain::Timer::with_state(
        cmd.task_id,
        timer
            .state()
            .clone()
            .with_remaining_seconds(new_timer_remaining),
    );
```

with:

```rust
    task_repo.update(task).await?;

    // The new task is bound to the timer in the Idle state — the previous
    // task's in-progress phase/remaining is NOT carried over. The user starts
    // the new task's pomodoro explicitly. `Timer::new` encodes this invariant
    // ("bound to task_id, starting in the Idle state").
    let new_timer = domain::Timer::new(cmd.task_id);
```

This deletes the now-unused `new_timer_remaining` binding and the `with_remaining_seconds` call. `task_repo.update(task)` is preserved (moved above the comment). No import changes are needed — `with_remaining_seconds` was a method, not an import.

- [ ] **Step 5: Run the tests and confirm they PASS**

Run: `cargo test -p infra --test main -- should_switch_active_task`
Expected: PASS — all four `switch_active_task` tests now see an Idle timer bound to task2.

- [ ] **Step 6: Run the full workspace test suite**

Run: `cargo test`
Expected: all pass. If a caller of `switch_active_task` (e.g. `progress_phase`, `complete_flow`) breaks, it is relying on the removed carry-state behavior — re-read that caller; per the spec, the double-reset at those call sites is harmless and any failure there is a test asserting the OLD behavior, which should be updated to expect Idle.

- [ ] **Step 7: Format and commit**

```bash
cargo fmt
git add core/usecases/src/task/switch_active_task.rs core/infra/tests/app/adv_timer.rs
git commit -m "fix(usecases/task): switch_active_task resets new task's timer to Idle

Switching the active task no longer carries the old timer phase/remaining
onto the new task. The new task is bound via Timer::new, leaving the
timer Idle (Stopped, 0 remaining) until the user starts it. Updates the
Working-path test and adds ShortBreak/LongBreak/Paused regression tests."
```

---

## Task 2: Same-task idempotency guard in `switch_active_task`

After Task 1, a no-op "switch" to the *currently running* task would silently stop the timer (because `Timer::new` resets to Idle) and churn task status (queue then re-activate). This task makes same-task switches a true no-op.

**Files:**
- Modify: `core/infra/tests/app/adv_timer.rs` (add 1 test)
- Modify: `core/usecases/src/task/switch_active_task.rs`

**Interfaces:**
- Produces: `switch_active_task` returning `Ok(())` early when `timer.task_id() == Some(cmd.task_id)`, leaving the running timer untouched.

- [ ] **Step 1: Write the failing test**

Append this test after the Task 1 tests in `core/infra/tests/app/adv_timer.rs`:

```rust
// Test 32: Switching to the task that is already active must be a no-op —
// a running timer must NOT be stopped by a redundant switch.
#[tokio::test]
async fn should_switch_active_task_to_same_task_is_noop() {
    let ctx = setup_ctx("should_switch_active_task_to_same_task_is_noop").await;

    let task1 = TaskBuilder::new()
        .name("Running task")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .build();
    ctx.task_repo
        .create(task1.clone())
        .await
        .expect("Failed to create task1");

    // Start a running work phase for task1.
    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd {
            task_id: Some(task1.id()),
        },
    )
    .await
    .expect("Failed to start task1");

    let running_remaining = get_timer(&ctx).await.state().remaining_seconds();
    assert_eq!(
        get_timer(&ctx).await.state().status(),
        TimerStatus::Running,
        "precondition: task1 should be running"
    );

    // Act: "switch" to the same task.
    let switch_result = usecases::task::switch_active_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        usecases::task::SwitchActiveTaskCmd {
            task_id: task1.id(),
            old_task_id: Some(task1.id()),
        },
    )
    .await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Assert: timer is STILL running, countdown unchanged.
    let timer_after = get_timer(&ctx).await;
    assert!(switch_result.is_ok(), "same-task switch should succeed");
    assert_eq!(timer_after.task_id(), Some(task1.id()));
    assert_eq!(
        timer_after.state().status(),
        TimerStatus::Running,
        "same-task switch must not stop a running timer"
    );
    assert_eq!(
        timer_after.state().remaining_seconds(),
        running_remaining,
        "countdown must be unchanged by a same-task switch"
    );
}
```

- [ ] **Step 2: Run the test and confirm it FAILS**

Run: `cargo test -p infra --test main -- should_switch_active_task_to_same_task_is_noop`
Expected: FAIL on `status == Running` — without the guard, Task 1's `Timer::new` resets the running timer to Idle.

- [ ] **Step 3: Implement the idempotency guard**

In `core/usecases/src/task/switch_active_task.rs`, locate the timer fetch (the line `let timer = timer_repo.get().await?;`, currently around line 36). Immediately after it, before the `let old_task_id = ...` block, insert:

```rust
    let timer = timer_repo.get().await?;

    // No-op: switching to the task that is already active must not stop a
    // running timer or churn task status.
    if timer.task_id() == Some(cmd.task_id) {
        return Ok(());
    }
```

(Anchor the edit on `let timer = timer_repo.get().await?;` rather than a line number — Task 1 may have shifted nearby lines.)

- [ ] **Step 4: Run the test and confirm it PASSES**

Run: `cargo test -p infra --test main -- should_switch_active_task_to_same_task_is_noop`
Expected: PASS.

- [ ] **Step 5: Run the full workspace test suite**

Run: `cargo test`
Expected: all pass.

- [ ] **Step 6: Format and commit**

```bash
cargo fmt
git add core/usecases/src/task/switch_active_task.rs core/infra/tests/app/adv_timer.rs
git commit -m "fix(usecases/task): make same-task switch a no-op in switch_active_task

Adds an idempotency guard: switching to the already-active task returns
early instead of resetting a running timer to Idle and churning task
status. Adds same-task no-op regression test."
```

---

## Task 3: Align strict `switch_task` to reset to Idle

`switch_task` (the strict variant that blocks running switches) still preserves the entire old state (`with_state(cmd.task_id, timer.state().clone())`), so a Paused timer leaks its phase onto the new task. Align it to `Timer::new` for invariant uniformity.

**Files:**
- Modify: `core/infra/tests/app/adv_timer.rs` (add 1 test)
- Modify: `core/usecases/src/task/switch_task.rs:86-90`

**Interfaces:**
- Produces: `switch_task` leaving the singleton timer as `Active { task_id: new, state: Idle }`, matching `switch_active_task`.

- [ ] **Step 1: Write the failing test**

Append this test after the Task 2 tests in `core/infra/tests/app/adv_timer.rs`:

```rust
// Test 33: Strict switch_task from a Paused timer resets the new task to
// Idle. validate_task_switch allows Paused (not running), so the Paused
// phase must not carry over to the new task.
#[tokio::test]
async fn should_switch_task_from_paused_resets_to_idle() {
    let ctx = setup_ctx("should_switch_task_from_paused_resets_to_idle").await;

    let task1 = TaskBuilder::new()
        .name("Paused task")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .build();
    ctx.task_repo
        .create(task1.clone())
        .await
        .expect("Failed to create task1");

    let task2 = TaskBuilder::new()
        .name("Next task")
        .max_sessions(4)
        .status(TaskStatus::Queued)
        .build();
    ctx.task_repo
        .create(task2.clone())
        .await
        .expect("Failed to create task2");

    // task1's timer is Paused — not running, so validate_task_switch allows it.
    let paused_timer = domain::Timer::with_state(
        task1.id(),
        TimerState::Paused {
            paused_from: Box::new(TimerState::Working {
                remaining_seconds: 600,
            }),
            remaining_seconds: 600,
        },
    );
    ctx.timer_repo
        .save(&paused_timer)
        .await
        .expect("Failed to save paused timer");

    let switch_result = usecases::task::switch_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        usecases::task::SwitchTaskCmd {
            task_id: task2.id(),
        },
    )
    .await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let timer_after = get_timer(&ctx).await;
    assert!(switch_result.is_ok(), "Failed to switch task");
    assert_eq!(timer_after.task_id(), Some(task2.id()));
    assert!(
        timer_after.state().is_idle(),
        "Timer should be Idle after switch_task, not a carried-over Paused state"
    );
}
```

- [ ] **Step 2: Run the test and confirm it FAILS**

Run: `cargo test -p infra --test main -- should_switch_task_from_paused_resets_to_idle`
Expected: FAIL on `is_idle()` — current `switch_task` preserves the Paused state onto task2.

- [ ] **Step 3: Implement the alignment**

In `core/usecases/src/task/switch_task.rs`, replace lines 86-90 (the "preserving the state" comment + `with_state` call):

```rust
    // Create a new timer for the new task, preserving the state from the old timer
    // This allows seamless task switching during active sessions
    let new_timer =
        domain::Timer::with_state(cmd.task_id, timer.state().clone());
```

with:

```rust
    // Bind the new task in the Idle state. The previous task's phase/remaining
    // is NOT carried over — invariant matching `switch_active_task`.
    let new_timer = domain::Timer::new(cmd.task_id);
```

The `Timer` import remains in use (it appears in the `validate_task_switch(task: &Task, timer: &Timer)` signature at line 13). `validate_task_switch`'s running-guard stays unchanged — it is an orthogonal, stricter precondition.

- [ ] **Step 4: Run the test and confirm it PASSES**

Run: `cargo test -p infra --test main -- should_switch_task_from_paused_resets_to_idle`
Expected: PASS.

- [ ] **Step 5: Run the full workspace test suite**

Run: `cargo test`
Expected: all pass.

- [ ] **Step 6: Format and commit**

```bash
cargo fmt
git add core/usecases/src/task/switch_task.rs core/infra/tests/app/adv_timer.rs
git commit -m "fix(usecases/task): switch_task resets new task's timer to Idle

Aligns the strict switch path with switch_active_task: the new task is
bound via Timer::new (Idle), so a Paused phase no longer leaks onto the
switched task. Keeps the existing running-guard. Adds Paused regression test."
```

---

## Self-Review (completed during planning)

- **Spec coverage:** Change 1 (switch_active_task reset) → Task 1. Change 2 (idempotency guard) → Task 2. Change 3 (switch_task alignment) → Task 3. Tests (update Working-path, add ShortBreak/LongBreak/Paused/same-task for switch_active_task, add Paused for switch_task) → Tasks 1-3. The two architectural "no"s (no event emission, no clear task_id) are Global Constraints, not tasks. Out-of-scope items (double-reset cleanup, tray sync, decoupling) are intentionally not implemented here. ✅
- **Placeholder scan:** no TBD/TODO; every code step contains full code. ✅
- **Type consistency:** `Timer::new(task_id) -> Timer` used identically in Tasks 1 and 3; `TimerStatus::Stopped` / `is_idle()` / `remaining_seconds() == 0` consistent with `state_machine.rs`. ✅
- **Ordering note:** Task 2's guard is anchored on `let timer = timer_repo.get().await?;` (content, not line number) because Task 1 shifts nearby lines. ✅
