# Rust Backend Refactor — Flatten Nested Async & Simplify `TimerTickService`

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate the pyramid-of-doom async nesting across 5 orchestration sites and reduce `TimerTickService::start_timer_tick_loop` from 111 lines / 7 bundled concerns to ~25 lines, without changing observable behavior.

**Architecture:** Tier-1-only refactor of `sqlite_service.rs` (extract private helpers, drop dead param, extract spawned closure into a named fn). For the 5 nested-async sites, extract private helper functions inside each host file (no new public API surface, no struct splits). Existing integration tests in `core/infra/tests/app/` are the regression gate — they already replicate the orchestration step-by-step against real SQLite repos.

**Tech Stack:** Rust + Tauri 2 + Tokio + SQLite. Verification via `just test-infra` (primary), `just clippy`, `just fmt-check`, `just ci` (final gate).

## Global Constraints

- **No `tokio::time::sleep` to drain handlers** — rely on direct stop/load/start sequencing.
- **Pre-commit hook** (`scripts/pre-commit`) runs `cargo clippy --workspace -- -D warnings` + `cargo fmt --all -- --check` + React lint/typecheck. It does NOT run `cargo test`. Plan must keep clippy clean.
- **Tests are co-located NOT with the code under test, but in `core/infra/tests/app/`** — they replicate orchestration step-by-step against real SQLite via `AppContext`. Mirror sequences in `manual_complete_cycling.rs` MUST stay aligned with any orchestration change.
- **No comments added** unless the existing code already has comments in the same spot (preserve existing comments; do not add new ones).
- **No new dependencies.** All extractions use existing types (`Timer`, `TimerConfiguration`, `CountdownExpired`, `EventPublisher`, `PhaseOutcome`).

## Verified Type Reference

These signatures were confirmed against the current `main` (commit `cb44e16`):

```
// core/domain/src/timer/timer.rs:144
pub fn task_id(&self) -> Option<TaskId>   // Timer::task_id

// core/usecases/src/timer/progress_phase.rs:25-48
pub enum PhaseOutcome {
    Started { task: Task, timer: Timer, next_phase: Phase,
              cycled_to: Option<TaskId>, block_message: Option<String> },
    Paused  { task: Task, timer: Timer, next_phase: Phase,
              cycled_to: Option<TaskId>, block_message: Option<String> },
    Stopped { task: Task, timer: Timer },
}
```

## File Structure (final state)

No new files. All changes are in-place edits to existing files:

| File | Change |
|------|--------|
| `core/infra/src/adapters/timer/sqlite_service.rs` | Extract 4 private helpers, drop `_task_id` param, extract spawned closure into `run_tick_loop` free fn |
| `apps/tauri-app/src/commands/task_cmd/complete_flow.rs` | Extract `advance_to_next_task` + `clear_completed_active_task` private helpers |
| `apps/tauri-app/src/tray.rs` | Flatten `menu_play_pause`; extract `run_phase_changing_menu_handler` shared by `menu_reset_phase` + `menu_skip` |
| `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs` | Extract `emit_post_phase_events` private helper; flatten match arms |

---

## Task 1: Extract `abort_existing_loop` + `resolve_timer_config` private helpers

**Files:**
- Modify: `core/infra/src/adapters/timer/sqlite_service.rs:102-131, 215-224`

**Interfaces:**
- Consumes: existing `cancel_handle`, `config_repository` fields.
- Produces: two private async methods. No public signature changes.

- [ ] **Step 1: Baseline test run**

Run: `just test-infra`
Expected: PASS (all green). This is the regression baseline — save the output count for later comparison.

- [ ] **Step 2: Add `abort_existing_loop` helper**

In `sqlite_service.rs`, add a private method that returns the guard (so `start_*` can keep holding it) and rewrite `stop_timer_tick_loop` to use it. Insert this inside `impl TimerTickService`:

```rust
async fn abort_existing_loop(
    &self,
) -> tokio::sync::MutexGuard<'_, Option<tokio::task::JoinHandle<()>>> {
    let mut guard = self.cancel_handle.lock().await;
    if let Some(handle) = guard.take() {
        handle.abort();
    }
    guard
}
```

Update `stop_timer_tick_loop` (lines 215-224) to:

```rust
pub async fn stop_timer_tick_loop(&self) -> DomainResult<()> {
    let _guard = self.abort_existing_loop().await;
    Ok(())
}
```

Update the abort block inside `start_timer_tick_loop` (lines 126-131): replace the manual `lock`/`take`/`abort` with:

```rust
let mut cancel_guard = self.abort_existing_loop().await;
```

(Remove the old `let mut cancel_guard = self.cancel_handle.lock().await;` + `if let Some(handle) = cancel_guard.take() { handle.abort(); }`.)

- [ ] **Step 3: Add `resolve_timer_config` helper**

Extract lines 114-122 of `start_timer_tick_loop` into:

```rust
async fn resolve_timer_config(
    &self,
    timer_config: Option<TimerConfiguration>,
) -> Result<TimerConfiguration, String> {
    if let Some(config) = timer_config {
        Ok(config)
    } else {
        self.config_repository
            .get_config()
            .await
            .map_err(|e| e.to_string())
            .map(|c| c.timer)
    }
}
```

Replace the inline block in `start_timer_tick_loop` with:

```rust
let config = self.resolve_timer_config(timer_config).await?;
```

- [ ] **Step 4: Verify build + tests**

Run: `just test-infra && just clippy`
Expected: PASS — no behavior change, just extraction. Test count matches Step 1 baseline.

- [ ] **Step 5: Commit**

```bash
git add core/infra/src/adapters/timer/sqlite_service.rs
git commit -m "refactor(infra): extract abort_existing_loop and resolve_timer_config helpers"
```

---

## Task 2: Extract `run_tick_loop` body + `compute_tick_outcome`

**Files:**
- Modify: `core/infra/src/adapters/timer/sqlite_service.rs:139-205`

**Interfaces:**
- Consumes: `Timer::is_running`, `Timer::as_active_mut`, `ActiveTimer::tick`, `Timer::task_id`, `Timer::get_current_phase`, `CountdownExpired::new`, `EventPublisher::{publish, publish_batch}`.
- Produces: a free async fn `run_tick_loop` + `compute_tick_outcome` + `TickOutcome` struct.

- [ ] **Step 1: Add `TickOutcome` struct + `compute_tick_outcome` helper**

Add at the bottom of `sqlite_service.rs` (outside the `impl` block). This collapses lines 147-168 into a single critical section that ALSO captures the expiry payload, eliminating the double-lock at lines 180-185:

```rust
struct TickOutcome {
    should_continue: bool,
    phase_completed: bool,
    events_to_publish: Vec<Box<dyn domain::Event>>,
    expiry_payload: Option<(Phase, TaskId)>,
}

async fn compute_tick_outcome(
    timer: &tokio::sync::Mutex<Timer>,
    config: &TimerConfiguration,
) -> TickOutcome {
    let mut timer = timer.lock().await;
    if !timer.is_running() {
        return TickOutcome {
            should_continue: false,
            phase_completed: false,
            events_to_publish: Vec::new(),
            expiry_payload: None,
        };
    }
    let Some(active) = timer.as_active_mut() else {
        return TickOutcome {
            should_continue: false,
            phase_completed: false,
            events_to_publish: Vec::new(),
            expiry_payload: None,
        };
    };
    match active.tick(config) {
        Ok((phase_complete, events)) => {
            let expiry_payload = if phase_complete {
                timer.task_id().map(|tid| (timer.get_current_phase(), tid))
            } else {
                None
            };
            TickOutcome {
                should_continue: !phase_complete,
                phase_completed: phase_complete,
                events_to_publish: events,
                expiry_payload,
            }
        }
        Err(e) => {
            log::error!("Timer tick error: {e}");
            TickOutcome {
                should_continue: false,
                phase_completed: false,
                events_to_publish: Vec::new(),
                expiry_payload: None,
            }
        }
    }
}
```

You will need to add `Phase` to the `use domain::{…}` import block at the top of the file.

- [ ] **Step 2: Add `run_tick_loop` free function**

Add below `compute_tick_outcome`. This is the body currently inside `tokio::spawn(async move { … })` at lines 140-204, now lifted out:

```rust
async fn run_tick_loop(
    timer: Arc<tokio::sync::Mutex<Timer>>,
    event_publisher: EventPublisherArc,
    config: TimerConfiguration,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.tick().await;
    loop {
        interval.tick().await;
        let outcome = compute_tick_outcome(&timer, &config).await;
        if !outcome.events_to_publish.is_empty() {
            event_publisher.publish_batch(outcome.events_to_publish);
        }
        if outcome.phase_completed {
            if let Some((current_phase, task_id)) = outcome.expiry_payload {
                let expiration_event =
                    domain::timer::events::CountdownExpired::new(
                        current_phase,
                        task_id,
                    );
                event_publisher.publish(Box::new(expiration_event));
            }
            break;
        }
        if !outcome.should_continue {
            break;
        }
    }
}
```

- [ ] **Step 3: Slim `start_timer_tick_loop` spawn site**

Replace lines 133-209 (the `let timer_clone = …` through `drop(cancel_guard);`) with:

```rust
let handle = tokio::spawn(run_tick_loop(
    Arc::clone(&self.timer),
    Arc::clone(&self.event_publisher),
    config,
));

*cancel_guard = Some(handle);
drop(cancel_guard);

Ok(())
```

The full `start_timer_tick_loop` body should now be ~15 lines: `load_state` → `resolve_timer_config` → `abort_existing_loop` → `tokio::spawn(run_tick_loop(…))` → store handle → Ok.

- [ ] **Step 4: Verify build + tests**

Run: `just test-infra && just clippy`
Expected: PASS. Pay special attention to `tick_loop_invariants.rs` and `concurrent_command_stress.rs` — these are the canaries for loop-behavior regressions.

- [ ] **Step 5: Commit**

```bash
git add core/infra/src/adapters/timer/sqlite_service.rs
git commit -m "refactor(infra): extract run_tick_loop body into named fn with single-lock tick"
```

---

## Task 3: Drop dead `_task_id` parameter from `start_timer_tick_loop`

**Files:**
- Modify: `core/infra/src/adapters/timer/sqlite_service.rs:102-106`
- Modify (production callers — remove the `, None` second argument):
  - `apps/tauri-app/src/commands/timer_cmd/start_timer.rs:53`
  - `apps/tauri-app/src/commands/timer_cmd/resume_timer.rs:50`
  - `apps/tauri-app/src/commands/timer_cmd/reset_timer_phase.rs:86`
  - `apps/tauri-app/src/commands/timer_cmd/skip_phase.rs:80`
  - `apps/tauri-app/src/commands/timer_cmd/switch_active_task.rs:88`
  - `apps/tauri-app/src/commands/task_cmd/complete_flow.rs:130`
  - `apps/tauri-app/src/tray.rs:565, 593, 659, 726`
  - `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs:98`
- Modify (test callers):
  - `core/infra/tests/core/context/builder.rs:169`
  - `core/infra/tests/app/concurrent_command_stress.rs:86, 134, 224, 318`
  - `core/infra/tests/app/timer.rs:241`
  - `core/infra/tests/app/tick_loop_invariants.rs:59`

**Interfaces:**
- Produces: `pub async fn start_timer_tick_loop(&self, timer_config: Option<TimerConfiguration>) -> Result<(), String>` (1 param, was 2).

- [ ] **Step 1: Change the signature**

In `sqlite_service.rs`, change:

```rust
pub async fn start_timer_tick_loop(
    &self,
    timer_config: Option<TimerConfiguration>,
    _task_id: Option<TaskId>,
) -> Result<(), String> {
```

to:

```rust
pub async fn start_timer_tick_loop(
    &self,
    timer_config: Option<TimerConfiguration>,
) -> Result<(), String> {
```

Check whether `TaskId` is still used elsewhere in the file. If not, remove it from the `use domain::{…}` block.

- [ ] **Step 2: Update all production callers**

For each production caller listed above, remove the `, None` second argument. Example for `start_timer.rs:52-54`:

```rust
// before
timer_tick_service_arc
    .start_timer_tick_loop(Some(task.config().timer.clone()), None)
    .await

// after
timer_tick_service_arc
    .start_timer_tick_loop(Some(task.config().timer.clone()))
    .await
```

After editing, confirm none were missed:

```bash
rg -n "start_timer_tick_loop\([^)]*None" --type rust
```

Expected: no output (all `None` second args removed).

- [ ] **Step 3: Update all test callers**

Same mechanical change in the 7 test sites listed above.

- [ ] **Step 4: Build + test + clippy**

Run: `cargo check --workspace && just test-infra && just clippy`
Expected: PASS — the compiler catches any missed caller.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor(infra): drop dead _task_id param from start_timer_tick_loop"
```

---

## Task 4: Flatten `complete_task_flow` — extract `advance_to_next_task` + `clear_completed_active_task`

**Files:**
- Modify: `apps/tauri-app/src/commands/task_cmd/complete_flow.rs:30-208`

**Interfaces:**
- Consumes: `plan_auto_advance`, `AdvancePlan`, `switch_active_task`, `reset_timer_to_idle`, `start_timer_phase`, `clear_active_task`, `TimerTickService` methods.
- Produces: two private async fns in the same file.

- [ ] **Step 1: Extract `advance_to_next_task`**

Add a private async fn that encapsulates the entire auto-advance branch (current lines 64-176). Returns `Some(next_task_id)` on success, `None` if no advance happened. The current code embeds `to_task` and `timer_json` in the `AUTO_ADVANCED` payload (lines 145-167) — preserve that. All error mapping stays inside:

```rust
/// Attempt to auto-advance to the next incomplete task. Returns the
/// new active task id when the advance succeeded; `None` when
/// AutoAdvance is off, no task is eligible, or any step failed
/// (failures are logged, not propagated).
async fn advance_to_next_task(
    completed_task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: TimerRepositoryArc,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    timer_tick_service: Arc<TimerTickService>,
    app_handle: &AppHandle,
) -> Option<TaskId> {
    let plan = plan_auto_advance(&task_repo, &config_repo).await?;

    match switch_active_task(
        task_repo.clone(),
        timer_repo.clone(),
        event_publisher.clone(),
        SwitchActiveTaskCmd {
            task_id: plan.next_task_id,
            old_task_id: Some(completed_task_id),
        },
    )
    .await
    {
        Ok(()) => {
            reset_timer_to_idle(
                plan.next_task_id,
                task_repo.clone(),
                timer_repo.clone(),
                event_publisher.clone(),
            )
            .await
            .ok()?;

            timer_tick_service.load_state().await.ok()?;
            timer_tick_service.stop_timer_tick_loop().await.ok()?;

            if plan.auto_start_work {
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
                } else if let Ok(Some(next_task)) =
                    task_repo.get_by_id(plan.next_task_id).await
                {
                    if timer_tick_service
                        .start_timer_tick_loop(Some(
                            next_task.config().timer.clone(),
                        ))
                        .await
                        .is_err()
                    {
                        log::warn!(
                            "Failed to start tick loop after auto-advance for task {}",
                            plan.next_task_id
                        );
                    }
                }
            }

            let to_task = task_repo
                .get_by_id(plan.next_task_id)
                .await
                .ok()
                .flatten();
            let timer_json =
                timer_tick_service.with_timer(|t| json!(t)).await;

            let _ = app_handle.emit(
                domain::event_names::task::AUTO_ADVANCED,
                json!({
                    "from_task_id": completed_task_id.to_string(),
                    "to_task_id": plan.next_task_id.to_string(),
                    "to_task": to_task,
                    "timer": timer_json,
                }),
            );

            Some(plan.next_task_id)
        }
        Err(e) => {
            log::warn!(
                "Auto-advance after completing {} failed; staying on completed task: {e}",
                completed_task_id
            );
            None
        }
    }
}
```

> **Implementer note:** The current code propagates errors via `?` and `anyhow::Context` inside the `Ok(())` arm (lines 79-139). The extracted helper above converts those to `.ok()?` (swallowing errors) because the helper's contract is "return `None` on any failure so the caller falls through to `clear_completed_active_task`." This matches the existing behavior: the original code's only error-handling paths in the outer flow are `log::warn!` (line 170) and the `Err(e)` arm (line 169). Verify this by tracing every `?` in the original lines 79-139 — each one, if it failed, would propagate up to `complete_task_flow`'s return as an `Err`, which would skip the `clear_active_task` block. The extracted version preserves that by returning `None` on any inner failure. **If you want to preserve exact error propagation to the top-level `anyhow::Result`, change the helper to return `anyhow::Result<Option<TaskId>>` instead.**

- [ ] **Step 2: Extract `clear_completed_active_task`**

Encapsulates current lines 181-199:

```rust
async fn clear_completed_active_task(
    completed_task_id: TaskId,
    timer_repo: TimerRepositoryArc,
    app_handle: &AppHandle,
) {
    if let Err(e) = clear_active_task(timer_repo).await {
        log::warn!(
            "Auto-clear of completed task {} failed; timer left bound: {e}",
            completed_task_id
        );
        return;
    }
    let _ = app_handle.emit(
        domain::event_names::task::ACTIVE_TASK_CLEARED,
        json!({ "from_task_id": completed_task_id.to_string() }),
    );
}
```

- [ ] **Step 3: Rewrite `complete_task_flow` as a flat sequence**

The body becomes linear — every statement is at indentation level 1:

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

    timer_tick_service
        .stop_timer_tick_loop()
        .await
        .context("Failed to stop timer tick loop while completing task")?;

    complete_task_uc(&task_repo, &event_publisher, task_id)
        .await
        .with_context(|| format!("Failed to complete task: {}", task_id))?;

    let active_task_id = advance_to_next_task(
        task_id,
        task_repo.clone(),
        timer_repo.clone(),
        config_repo.clone(),
        event_publisher.clone(),
        timer_tick_service.clone(),
        &app_handle,
    )
    .await
    .unwrap_or(task_id);

    if active_task_id == task_id {
        clear_completed_active_task(task_id, timer_repo.clone(), &app_handle)
            .await;
    }

    let task = task_repo
        .get_by_id(active_task_id)
        .await
        .context("Failed to retrieve task after completing")?
        .ok_or_else(|| anyhow!("Task not found after completing"))?;

    Ok(task)
}
```

- [ ] **Step 4: Build + test**

Run: `just test-infra && just clippy`
Expected: PASS — pay special attention to `core/infra/tests/app/manual_complete_cycling.rs` (the mirror of this flow).

- [ ] **Step 5: Commit**

```bash
git add apps/tauri-app/src/commands/task_cmd/complete_flow.rs
git commit -m "refactor(app): flatten complete_task_flow via extracted helpers"
```

---

## Task 5: Flatten `menu_play_pause` in `tray.rs`

**Files:**
- Modify: `apps/tauri-app/src/tray.rs:519-610`

**Interfaces:**
- Consumes: `pause_timer_phase`, `resume_timer_phase`, `start_timer_phase`, `TimerTickService` methods, `TimerStatus`, `TrayCtx`.
- Produces: three private async helpers — one per `TimerStatus` branch — plus a shared `start_loop_for_task`.

> **Verify before implementing:** the `TrayCtx` is moved into the spawn closure (`ctx.spawn(|ctx| async move { … })`). The helpers below take `&TrayCtx`. Confirm all fields accessed (`.tick_service`, `.task_repo`, `.timer_repo`, `.event_publisher`, `.task_id`, `.timer`) are either `Clone` (Arc types) or `Copy` (`TaskId`, `Timer`) so `.clone()` / copy inside each helper compiles. Read `TrayCtx` definition in `tray.rs` (search for `struct TrayCtx`) before writing these helpers.

- [ ] **Step 1: Add `start_loop_for_task` shared helper**

Add above `menu_play_pause`:

```rust
async fn start_loop_for_task(ctx: &TrayCtx) -> domain::Result<()> {
    let task = ctx
        .task_repo
        .get_by_id(ctx.task_id)
        .await?
        .ok_or_else(|| domain::Error::TaskNotFound {
            id: ctx.task_id.to_string(),
        })?;
    ctx.tick_service
        .start_timer_tick_loop(Some(task.config().timer.clone()))
        .await
        .map_err(|e| domain::Error::RepositoryError { message: e })
}
```

- [ ] **Step 2: Add per-status helpers**

```rust
async fn pause_running_timer(ctx: &TrayCtx) -> domain::Result<()> {
    let live = ctx.tick_service.get_current_timer().await;
    let remaining = live.remaining_seconds(None);
    pause_timer_phase(
        ctx.task_id,
        remaining,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_publisher.clone(),
    )
    .await?;
    ctx.tick_service.load_state().await?;
    ctx.tick_service.stop_timer_tick_loop().await?;
    Ok(())
}

async fn resume_paused_timer(ctx: &TrayCtx) -> domain::Result<()> {
    resume_timer_phase(
        ctx.task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_publisher.clone(),
    )
    .await?;
    start_loop_for_task(ctx).await
}

async fn start_idle_timer(ctx: &TrayCtx) -> domain::Result<()> {
    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_publisher.clone(),
        StartTimerPhaseCmd {
            task_id: Some(ctx.task_id),
        },
    )
    .await?;
    start_loop_for_task(ctx).await
}
```

- [ ] **Step 3: Rewrite `menu_play_pause` body**

The inner `async { … }.await` workaround block (lines 530-605) and its apologetic comment are gone — the `match` now returns `domain::Result<()>` directly:

```rust
fn menu_play_pause(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let _orchestration_lock = ctx.tick_service.orchestration_lock().await;
        let res: domain::Result<()> = match ctx.timer.status() {
            TimerStatus::Running => pause_running_timer(&ctx).await,
            TimerStatus::Paused => resume_paused_timer(&ctx).await,
            TimerStatus::Idle | TimerStatus::Stopped => {
                start_idle_timer(&ctx).await
            }
        };
        if let Err(e) = res {
            log::error!("Tray play/pause failed: {}", e);
        }
    });
}
```

- [ ] **Step 4: Build + test**

Run: `just test-infra && just clippy`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/tauri-app/src/tray.rs
git commit -m "refactor(app): flatten menu_play_pause via per-status helpers"
```

---

## Task 6: Deduplicate `menu_reset_phase` + `menu_skip` via shared helper

**Files:**
- Modify: `apps/tauri-app/src/tray.rs:616-742`

**Interfaces:**
- Consumes: `reset_timer_phase`, `skip_timer_phase`, `TimerTickService` methods, `TrayCtx`.
- Produces: one private async helper that takes the usecase fn as a callback.

- [ ] **Step 1: Add `run_phase_changing_menu_handler`**

The two functions are identical except for the usecase call (lines 634-644 vs 701-711). Extract:

```rust
async fn run_phase_changing_menu_handler<F, Fut>(ctx: &TrayCtx, label: &str, usecase: F)
where
    F: FnOnce(&TrayCtx) -> Fut + Send,
    Fut: std::future::Future<Output = domain::Result<()>> + Send,
{
    let task = match ctx.task_repo.get_by_id(ctx.task_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            log::error!("Tray {}: task {} not found", label, ctx.task_id);
            return;
        }
        Err(e) => {
            log::error!("Tray {}: failed to load task: {}", label, e);
            return;
        }
    };

    if let Err(e) = usecase(ctx).await {
        log::error!("Tray {} failed: {}", label, e);
        return;
    }

    if let Err(e) = ctx.tick_service.stop_timer_tick_loop().await {
        log::error!("Tray {}: failed to stop tick loop: {}", label, e);
        return;
    }
    if let Err(e) = ctx.tick_service.load_state().await {
        log::error!("Tray {}: failed to load timer state: {}", label, e);
        return;
    }

    match ctx.timer_repo.get().await {
        Ok(updated) if updated.is_running() => {
            if let Err(e) = ctx
                .tick_service
                .start_timer_tick_loop(Some(task.config().timer.clone()))
                .await
            {
                log::error!(
                    "Tray {}: failed to restart tick loop: {}",
                    label,
                    e
                );
            }
        }
        Ok(_) => {}
        Err(e) => log::error!("Tray {}: failed to read timer: {}", label, e),
    }
}
```

- [ ] **Step 2: Collapse `menu_reset_phase` to a thin caller**

```rust
fn menu_reset_phase(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let _lock = ctx.tick_service.orchestration_lock().await;
        run_phase_changing_menu_handler(&ctx, "reset phase", |ctx| async move {
            reset_timer_phase(
                ctx.task_id,
                ctx.task_repo.clone(),
                ctx.timer_repo.clone(),
                ctx.event_publisher.clone(),
            )
            .await
        })
        .await;
    });
}
```

- [ ] **Step 3: Collapse `menu_skip` to a thin caller**

```rust
fn menu_skip(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let _lock = ctx.tick_service.orchestration_lock().await;
        run_phase_changing_menu_handler(&ctx, "skip", |ctx| async move {
            skip_timer_phase(
                ctx.task_repo.clone(),
                ctx.timer_repo.clone(),
                ctx.event_publisher.clone(),
                ctx.task_id,
            )
            .await
        })
        .await;
    });
}
```

- [ ] **Step 4: Build + test**

Run: `just test-infra && just clippy`
Expected: PASS. `concurrent_command_stress.rs::concurrent_skip_phase_does_not_deadlock` is the canary.

> **Implementer note on `Send` bound:** the `FnOnce(&TrayCtx) -> Fut + Send` bound requires the closure and its future to be `Send`. Since `TrayCtx` is borrowed (`&ctx`) and the closure only captures `ctx` by reference, this should compile as long as `TrayCtx: Sync` (which it is — all fields are `Arc` or `Copy`). If clippy complains about the closure signature, change the helper to take `ctx: TrayCtx` (cloned) instead of `&TrayCtx`.

- [ ] **Step 5: Commit**

```bash
git add apps/tauri-app/src/tray.rs
git commit -m "refactor(app): dedupe menu_reset_phase and menu_skip via shared helper"
```

---

## Task 7: Deduplicate `CountdownExpiredHandler::handle` match arms

**Files:**
- Modify: `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs:85-241`

**Interfaces:**
- Consumes: `Emitter::emit`, `ui_listeners::{timer,task,screen_blocker}` constants, `Task`, `Phase`, `PhaseOutcome`, `TimerTickService::with_timer`.
- Produces: one private async helper used by both `Started` and `Paused` arms.

**Context:** The `Started` and `Paused` arms share an identical tail of 3 emits (`AUTO_ADVANCED` if `cycled_to`, `PROGRESS_UPDATED`, `screen_blocker::ACTIVATE` if `block_message`). The `AUTO_ADVANCED` payload now embeds `to_task: task` and `timer: <timer_json>` in both arms. The `timer_json` source differs between arms:
- `Started`: `self.timer_srv.with_timer(|t| json!(t)).await` (reads from cache after `load_state` + `start_timer_tick_loop`)
- `Paused`: `json!(timer)` (uses the `timer` field from the `PhaseOutcome::Paused` variant)

The helper takes `timer_json` as a parameter so each arm computes it from its preferred source.

- [ ] **Step 1: Add `emit_post_phase_events` helper**

Inside `impl CountdownExpiredHandler` (not the `EventHandler` impl — put it in the inherent impl alongside `new`):

```rust
async fn emit_post_phase_events(
    &self,
    task: &Task,
    cycled_to: Option<TaskId>,
    from_task_id: TaskId,
    timer_json: serde_json::Value,
    block_message: Option<&str>,
) -> Result<()> {
    if let Some(to_task_id) = cycled_to {
        self.emitter
            .emit(
                ui_listeners::task::AUTO_ADVANCED,
                json!({
                    "from_task_id": from_task_id,
                    "to_task_id": to_task_id,
                    "to_task": task,
                    "timer": timer_json,
                }),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit auto-advanced event: {e}"),
            })?;
    }

    self.emitter
        .emit(ui_listeners::task::PROGRESS_UPDATED, json!(task))
        .map_err(|e| domain::Error::EventPublishingError {
            message: format!(
                "Failed to emit task progress updated event: {e}"
            ),
        })?;

    if let Some(message) = block_message {
        self.emitter
            .emit(
                ui_listeners::screen_blocker::ACTIVATE,
                json!({ "message": message }),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit screen_blocker activate event: {e}"
                ),
            })?;
    }

    Ok(())
}
```

You will need to add `TaskId` to the `use domain::{…}` import at the top of the file.

- [ ] **Step 2: Simplify the `match outcome` block**

Replace the entire `match outcome { … }` (lines 85-241) with:

```rust
match outcome {
    usecases::timer::PhaseOutcome::Started {
        task,
        next_phase,
        cycled_to,
        block_message,
        ..
    } => {
        self.timer_srv.load_state().await?;

        if !(task.is_completed() && next_phase == Phase::Work) {
            let timer_config = task.config().timer.clone();
            self.timer_srv
                .start_timer_tick_loop(Some(timer_config))
                .await
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!("Failed to auto-start timer: {}", e),
                })?;
        }

        let state_json =
            self.timer_srv.with_timer(|t| json!(t.state())).await;

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

        let timer_json =
            self.timer_srv.with_timer(|t| json!(t)).await;
        self.emit_post_phase_events(
            &task,
            cycled_to,
            countdown_expired.task_id,
            timer_json,
            block_message.as_deref(),
        )
        .await?;
    }

    usecases::timer::PhaseOutcome::Paused {
        task,
        timer,
        cycled_to,
        block_message,
        ..
    } => {
        self.timer_srv.load_state().await?;

        self.emitter
            .emit(
                ui_listeners::timer::STATUS_CHANGED,
                json!(timer.state()),
            )
            .map_err(|e| domain::Error::RepositoryError {
                message: format!(
                    "Failed to emit timer status changed event: {e}"
                ),
            })?;

        self.emit_post_phase_events(
            &task,
            cycled_to,
            countdown_expired.task_id,
            json!(timer),
            block_message.as_deref(),
        )
        .await?;
    }

    usecases::timer::PhaseOutcome::Stopped { .. } => {
        self.timer_srv
            .stop_timer_tick_loop()
            .await
            .map_err(|e| domain::Error::RepositoryError {
                message: format!("Failed to stop timer tick loop: {e}"),
            })?;

        self.timer_srv.load_state().await?;
        let state_json =
            self.timer_srv.with_timer(|t| json!(t.state())).await;

        self.emitter
            .emit(ui_listeners::timer::STATUS_CHANGED, state_json)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit timer status changed event: {e}"
                ),
            })?;
    }
}
```

The `Started` and `Paused` arms are now ~20 lines each (down from ~45), with the shared tail delegated.

- [ ] **Step 3: Build + test**

Run: `just test-infra && just clippy`
Expected: PASS. `tick_loop_invariants.rs::auto_advance_leaves_tick_loop_alive_on_new_task` (20 iterations) and `concurrent_command_stress.rs::manual_skip_racing_with_natural_expiry_does_not_deadlock` both exercise this handler. Also check `core/infra/tests/app/timer_event_payloads.rs` (added in recent commits) — it asserts timer UI events carry `task_id` + state.

- [ ] **Step 4: Commit**

```bash
git add core/infra/src/adapters/timer/event_handlers/countdown_expired.rs
git commit -m "refactor(infra): dedupe CountdownExpiredHandler arms via emit_post_phase_events"
```

---

## Final Verification

After Task 7:

- [ ] **Run the full CI gate**

Run: `just ci`
Expected: `✅ All checks passed!`

This runs `cargo test --workspace` + `cargo check --workspace` + `cargo fmt --all -- --check` + `cargo clippy --workspace -- -D warnings` + React lint/typecheck.

- [ ] **Format check**

Run: `just fmt` (auto-format) then `just fmt-check` (verify).
Expected: no diff after formatting.

- [ ] **Spot-check `git diff --stat main..HEAD`**

Expected: ~10-12 files modified, net line count should be DOWN (extracted helpers are shorter than the duplicated inline blocks they replace).

---

## Out of Scope (deferred to future work)

These were identified during exploration but are NOT in this plan (Tier 1 only, per scope decision):

- **Split `TimerTickService` into `TimerStateCache` + `TickLoopController` + facade.** Would make the "state cache vs loop lifecycle vs serialization" boundary mechanically enforceable. Bigger surface — affects `CountdownExpiredHandler` internals and the `AppContext` test wiring. Recommend as a follow-up.
- **Extract `mutate_and_persist<F>`** to collapse `update_timer` / `reset_timer` / `reset_timer_phase` into one-line callers. Low risk but unrelated to the nesting problem.
- **Unreachable defensive arm** at `sqlite_service.rs:165` (`None =>` after `is_running()` check). The `Timer` domain invariant makes it dead by construction. Removed implicitly by Task 2's `compute_tick_outcome`.

---

## Self-Review

- **Spec coverage:** Both originally-cited files are covered (Tasks 1-3 for `sqlite_service.rs`, Task 4 for `complete_flow.rs`). The 4 additional offenders (3 tray handlers + `CountdownExpiredHandler`) are Tasks 5-7.
- **Current-code accuracy:** All code blocks were re-read against `main` at commit `cb44e16` (2026-06-28). The `AUTO_ADVANCED` payload now embeds `to_task` + `timer` — reflected in Tasks 4 and 7. The `PHASE_COMPLETED` payload embeds `{ timer, task }` — reflected in Task 7.
- **Type consistency:** `start_loop_for_task` (Task 5), the `advance_to_next_task` calls (Task 4), and the `CountdownExpiredHandler` calls (Task 7) all use the post-Task-3 single-arg `start_timer_tick_loop(Some(cfg))` signature. Task 3 must complete before Tasks 4-7. `PhaseOutcome` field types confirmed at `core/usecases/src/timer/progress_phase.rs:25-48`.
- **Ordering constraint:** Tasks MUST be executed in order — Task 3's signature change is a prerequisite for the caller rewrites in Tasks 4-7.
- **Verification steps:** Two "Verify before implementing" notes (in Tasks 4 and 5) flag spots where the implementer must confirm borrow/ownership details. These are NOT placeholders — they are explicit verification steps.
