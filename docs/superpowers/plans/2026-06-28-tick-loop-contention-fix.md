# Tick-Loop Contention Fix (macOS M1 Freeze) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate the intermittent UI freeze reproduced reliably on macOS M1 when rapidly invoking mutating timer commands (skip / pause / resume / etc.), by (C) dropping the `timer` mutex across the SQLite write in `save_state` and (A) adding a backend re-entry guard that serializes mutating orchestrations.

**Architecture:** The freeze is timing-sensitive — it widens on slower hardware. Rust-level stress tests on a fast Linux box (10× faster than M1) cannot reproduce it; the same code path on M1 hits the race every time. Two structural defects expose the window:

1. **`TimerTickService::save_state` holds the `timer` tokio Mutex across the repository write `.await`** (`sqlite_service.rs:41-48`). On slow disk the write takes 10s of ms, the lock stays held, and the tick-loop task or a concurrent orchestration queues on it. **Fix C** clones the timer under the lock, drops the guard, then writes — eliminating the cross-`await` hold with zero behavior change.

2. **No backend re-entry guard.** Every mutating entry point (Tauri command, tray handler, `CountdownExpiredHandler`) drives the same stop/load/start sequence against the shared `TimerTickService`. Two overlapping orchestrations race on the DB and the tick loop. The frontend `isBusy` guard hides this for UI clicks but cannot prevent tray-vs-UI or skip-vs-natural-expiry collisions. **Fix A** adds a `tokio::sync::Mutex<()>` (`orchestration_lock`) on `TimerTickService`; each mutating entry point holds the guard for its full body. Internal methods (`start_timer_tick_loop` / `stop_timer_tick_loop` / `load_state`) do NOT acquire the lock — they assume the caller holds it. This avoids recursion and keeps the hot path cheap.

Fix B (`spawn_blocking` for diesel calls) is intentionally deferred. If A+C don't fully resolve the freeze on M1, B is the next increment; the plan ends with a verification step on macOS that gates that decision.

**Tech Stack:** Rust, Tauri 2, tokio, async-trait, SQLite, existing `InMemoryEventBus` (unchanged). Tests are integration tests under `core/infra/tests/app/`.

## Global Constraints

- **Dependency rule:** dependencies point inward toward `domain`. No new reference to `infra::adapters::TimerTickService` from `core/usecases/**` or `core/domain/**`. Fix A's lock lives on `TimerTickService` (infra) and is acquired only by app/infra-layer entry points.
- **No new deps:** do not add Cargo dependencies. `tokio::sync::Mutex` is already in scope.
- **Event bus stays fire-and-forget:** do not modify `mem_event_bus.rs`. Do not make `EventPublisher::publish` async. (`CountdownExpiredHandler::handle` is an async handler that may acquire the lock — that's fine, it's not the publisher.)
- **No sleeps:** do not add `tokio::time::sleep` to "drain" handlers. The orchestration lock makes ordering deterministic.
- **Sequencing rule unchanged:** when an orchestration needs both STOP and START, they must still be `await`-ed sequentially in that order (`stop` → `load` → `start`).
- **`CountdownExpiredHandler` is still an orchestrator, not a UI-only emitter.** Per CLAUDE.md it is the legitimate exception that may drive the tick loop. Fix A extends that to "may acquire `orchestration_lock`."
- **Style:** match existing conventions (anyhow `Context`, `infra::commands::...` log prefixes, `#[tauri::command(rename_all = "snake_case")]`). Run `cargo fmt` before each commit.
- **Each task ends with `cargo test -p infra` green for the workspace and a commit.**
- **Verification on macOS M1 is the final gate.** On Linux, Fix C's failing test reproduces the bug; A's primitive test passes trivially. Only M1 can confirm the end-to-end freeze is gone.

## File Structure

| File | Responsibility after this plan |
|------|--------------------------------|
| `core/infra/src/adapters/timer/sqlite_service.rs` | (Task 1) `save_state` drops lock before write. (Task 2) Adds `orchestration_lock: tokio::sync::Mutex<()>` field, updates `new()`, adds `pub async fn orchestration_lock(&self) -> tokio::sync::MutexGuard<'_, ()>`. |
| `core/infra/tests/app/tick_loop_mutex_contention.rs` (new) | (Task 1) Failing test asserting `save_state` does not hold the `timer` mutex during the repo write. |
| `core/infra/tests/app/orchestration_lock.rs` (new) | (Task 2) Test asserting `orchestration_lock` serializes concurrent callers (max concurrency = 1). |
| `core/infra/tests/app/mod.rs` | Adds module declarations for the two new test files. |
| `apps/tauri-app/src/commands/timer_cmd/skip_phase.rs` | (Task 3) Acquires `orchestration_lock` at top of body. |
| `apps/tauri-app/src/commands/timer_cmd/start_timer.rs` | (Task 3) Acquires `orchestration_lock` at top of body. |
| `apps/tauri-app/src/commands/timer_cmd/pause_timer.rs` | (Task 3) Acquires `orchestration_lock` at top of body. |
| `apps/tauri-app/src/commands/timer_cmd/resume_timer.rs` | (Task 3) Acquires `orchestration_lock` at top of body. |
| `apps/tauri-app/src/commands/timer_cmd/reset_timer.rs` | (Task 3) Acquires `orchestration_lock` at top of body. |
| `apps/tauri-app/src/commands/timer_cmd/reset_timer_phase.rs` | (Task 3) Acquires `orchestration_lock` at top of body. |
| `apps/tauri-app/src/commands/timer_cmd/switch_active_task.rs` | (Task 3) Acquires `orchestration_lock` at top of body. |
| `apps/tauri-app/src/tray.rs` | (Task 3) Acquires `orchestration_lock` at top of `menu_play_pause`, `menu_reset_phase`, `menu_skip`, `menu_reset_task` async bodies. |
| `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs` | (Task 3) Acquires `orchestration_lock` at top of `handle()` before the `progress_phase` call. |
| `core/infra/tests/app/concurrent_command_stress.rs` | Unchanged. Existing stress tests are the regression net for Task 3 — they must still pass after the lock is applied. |

**Unchanged on purpose:** `core/usecases/**`, `core/domain/**`, `mem_event_bus.rs`, `apps/tauri-app/src/commands/timer_cmd/get_timer_state.rs` (read-only, no lock), `apps/tauri-app/src/commands/timer_cmd/update_timer_secs.rs` (out of the freeze path; the React UI does not call it).

---

## Task 1: Fix C — Drop `timer` mutex across `save_state` DB write

**Files:**
- Modify: `core/infra/src/adapters/timer/sqlite_service.rs:41-48` (the `save_state` method)
- Create: `core/infra/tests/app/tick_loop_mutex_contention.rs`
- Modify: `core/infra/tests/app/mod.rs`

**Interfaces:**
- Consumes: `TimerTickService::new` (existing constructor signature — Task 1 does NOT change it).
- Produces: behavior change only. No signature change. Task 2 will add a new field to the struct.

**Why first:** smallest, most-targeted change. Directly removes the cross-`await` mutex hold that widens on M1's slower disk. If this alone fixes the freeze on M1, Tasks 2–3 are still worth doing as defense-in-depth but become lower priority.

- [ ] **Step 1: Write the failing test**

Create `core/infra/tests/app/tick_loop_mutex_contention.rs`:

```rust
//! Regression test for the macOS-M1 freeze root cause.
//!
//! `TimerTickService::save_state` used to hold the `timer` tokio Mutex across
//! the repository write `.await`. On slow disk (M1) this blocked the tick-loop
//! task and any concurrent orchestration that needed the timer, producing an
//! intermittent but reliable freeze. The fix is to clone the timer under the
//! lock, drop the guard, then write.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use domain::timer::{Error as TimerError, Result as TimerResult};
use domain::{ConfigRepository, Timer, TimerRepository};
use infra::adapters::events::mem_event_bus::EventPublisherArc;
use infra::adapters::timer::TimerTickService;
use tokio::sync::Notify;

/// Fake repository whose `save` blocks on a `Notify` until released by the
/// test. Lets us observe whether the timer mutex is held *during* the write.
struct BlockingSaveRepo {
    entered: Arc<Notify>,
    release: Arc<Notify>,
}

#[async_trait]
impl TimerRepository for BlockingSaveRepo {
    async fn get(&self) -> TimerResult<Timer> {
        Ok(Timer::idle())
    }

    async fn save(&self, _timer: &Timer) -> TimerResult<()> {
        self.entered.notify_one();
        self.release.notified().await;
        Ok(())
    }
}

/// Minimal config repo stub — the test never calls it, but `TimerTickService::new`
/// requires it.
struct StubConfigRepo;

#[async_trait]
impl ConfigRepository for StubConfigRepo {
    async fn get_config(&self) -> domain::Result<domain::Config> {
        Ok(domain::Config::default())
    }
    async fn save_config(
        &self,
        _config: &domain::Config,
    ) -> domain::Result<()> {
        Ok(())
    }
    async fn config_exists(&self) -> domain::Result<bool> {
        Ok(true)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn save_state_does_not_hold_timer_mutex_during_write() {
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());

    let timer_repo: Arc<dyn TimerRepository + Send + Sync> =
        Arc::new(BlockingSaveRepo {
            entered: entered.clone(),
            release: release.clone(),
        });
    let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
        Arc::new(StubConfigRepo);
    let event_publisher: EventPublisherArc =
        Arc::new(infra::adapters::events::mem_event_bus::InMemoryEventBus::new());

    let svc = Arc::new(TimerTickService::new(
        event_publisher,
        timer_repo,
        config_repo,
    ));

    // Start save_state in the background. It will enter the repo's `save`,
    // notify us, then block on `release`.
    let svc_clone = Arc::clone(&svc);
    let save_task =
        tokio::spawn(async move { svc_clone.save_state().await });

    // Wait until `save` is in flight.
    entered.notified().await;

    // While `save_state` is still awaiting the repo, try to acquire the timer
    // mutex via `with_timer`. With Fix C applied, this succeeds immediately.
    // Without Fix C (mutex held across the write), this times out.
    let probe = tokio::time::timeout(
        Duration::from_millis(200),
        svc.with_timer(|_t| ()),
    )
    .await;

    assert!(
        probe.is_ok(),
        "timer mutex must be releasable while save_state awaits the repository \
         write. Holding it across the `.await` is the macOS-M1 freeze root cause."
    );

    // Let the background save complete so the test can tear down cleanly.
    release.notify_one();
    save_task.await.unwrap().expect("save_state returned err");
}
```

In `core/infra/tests/app/mod.rs`, after the existing `mod tick_loop_invariants;` line, add:

```rust
mod tick_loop_mutex_contention;
```

- [ ] **Step 2: Run the test and verify it FAILS (hangs/probes the held lock)**

Run: `cargo test -p infra --test main -- tick_loop_mutex_contention --nocapture`
Expected: FAIL. The 200ms probe times out because `save_state` is still holding the `timer` mutex across the `repo.save(...).await`. The assertion fires with the "must be releasable" message.

> If the test passes on master, it means `save_state` no longer holds the lock across the await — someone may have already applied Fix C. Inspect `sqlite_service.rs:41-48` to confirm; do not proceed with Task 1 if the fix is already in.

- [ ] **Step 3: Apply Fix C**

In `core/infra/src/adapters/timer/sqlite_service.rs`, replace the `save_state` method (currently lines 41-48):

```rust
    pub async fn save_state(&self) -> DomainResult<()> {
        let timer_guard = self.timer.lock().await;
        self.timer_repository.save(&timer_guard).await.map_err(|e| {
            Error::RepositoryError {
                message: e.to_string(),
            }
        })
    }
```

with:

```rust
    /// Persist the in-memory timer to the repository.
    ///
    /// Acquires `timer` only long enough to clone a snapshot, then drops the
    /// guard BEFORE the repository write. Holding the mutex across the
    /// `.await` wedges the tick-loop task and any concurrent orchestration on
    /// slow disk (macOS M1 reproducibly froze here).
    pub async fn save_state(&self) -> DomainResult<()> {
        let timer_snapshot = {
            let guard = self.timer.lock().await;
            guard.clone()
        };
        self.timer_repository
            .save(&timer_snapshot)
            .await
            .map_err(|e| Error::RepositoryError {
                message: e.to_string(),
            })
    }
```

`Timer` is `Clone`, so the snapshot is cheap and correct. There is no behavior change for callers — they still observe "save completed" only after the write succeeds.

- [ ] **Step 4: Run the test and verify it PASSES**

Run: `cargo test -p infra --test main -- tick_loop_mutex_contention --nocapture`
Expected: PASS. The probe acquires the timer mutex within 200ms while `save_state` is still blocked on `release.notified()`.

- [ ] **Step 5: Run the full infra test suite — no regressions**

Run: `cargo test -p infra`
Expected: all tests pass. Pay attention to anything under `tests/app/` that exercises `save_state` indirectly (`update_timer`, `reset_timer`, `reset_timer_phase`).

- [ ] **Step 6: Commit**

```bash
git add core/infra/src/adapters/timer/sqlite_service.rs \
        core/infra/tests/app/tick_loop_mutex_contention.rs \
        core/infra/tests/app/mod.rs
git commit -m "fix(infra/timer): drop timer mutex across save_state DB write

save_state used to hold the tokio Mutex across the repository .await. On
slow disk (macOS M1) the write takes long enough that the tick-loop task
or a concurrent orchestration queues on the mutex, producing the
intermittent but reliable UI freeze. Fix: clone a snapshot under the
lock, drop the guard, then write.

Adds tick_loop_mutex_contention regression test that hangs without the
fix (probe cannot acquire the timer mutex while save_state awaits the
repo)."
```

---

## Task 2: Add `orchestration_lock` primitive to `TimerTickService`

**Files:**
- Modify: `core/infra/src/adapters/timer/sqlite_service.rs` (struct fields, `new()`, new method)
- Create: `core/infra/tests/app/orchestration_lock.rs`
- Modify: `core/infra/tests/app/mod.rs`

**Interfaces:**
- Consumes: nothing new.
- Produces: `pub async fn orchestration_lock(&self) -> tokio::sync::MutexGuard<'_, ()>` on `TimerTickService`. Task 3 callers consume this.

- [ ] **Step 1: Write the failing test**

Create `core/infra/tests/app/orchestration_lock.rs`:

```rust
//! Tests for the backend re-entry guard that serializes mutating timer
//! orchestrations (Tauri commands, tray handlers, CountdownExpiredHandler).
//!
//! Without serialization, two overlapping orchestrations race on the shared
//! `TimerTickService` and the singleton timer row. On fast hardware the
//! window is microseconds wide and rarely hit; on macOS M1 it is reliable.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use infra::adapters::events::mem_event_bus::InMemoryEventBus;
use infra::adapters::timer::TimerTickService;

use crate::core::context::AppContext;

/// 10 concurrent callers of `orchestration_lock` must run strictly one at a
/// time. The counter increments on entry, decrements on exit, and tracks the
/// max value seen — which must be 1.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn orchestration_lock_serializes_concurrent_callers() {
    let ctx = AppContext::with_name(Some(
        "orchestration_lock_serializes_concurrent_callers",
    ))
    .await
    .expect("Failed to build test context");
    let svc = ctx.timer_tick_service.clone();

    let active = Arc::new(AtomicU32::new(0));
    let max_seen = Arc::new(AtomicU32::new(0));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let svc = svc.clone();
        let active = active.clone();
        let max_seen = max_seen.clone();
        handles.push(tokio::spawn(async move {
            let _guard = svc.orchestration_lock().await;
            let now = active.fetch_add(1, Ordering::SeqCst) + 1;
            // fetch_max via compare-exchange loop
            let mut cur = max_seen.load(Ordering::SeqCst);
            while now > cur {
                match max_seen.compare_exchange(
                    cur,
                    now,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(actual) => cur = actual,
                }
            }
            // Hold the guard for a bit so concurrency would be detectable.
            tokio::time::sleep(Duration::from_millis(20)).await;
            active.fetch_sub(1, Ordering::SeqCst);
        }));
    }
    for h in handles {
        h.await.expect("worker panicked");
    }

    assert_eq!(
        max_seen.load(Ordering::SeqCst),
        1,
        "orchestration_lock must serialize callers — saw concurrent execution"
    );
}

/// The guard returned by `orchestration_lock` releases on drop. Once a caller
/// completes, the next waiter proceeds immediately.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn orchestration_lock_releases_on_drop() {
    let ctx = AppContext::with_name(Some("orchestration_lock_releases_on_drop"))
        .await
        .expect("Failed to build test context");
    let svc = ctx.timer_tick_service.clone();

    // First caller acquires and releases quickly.
    {
        let _g = svc.orchestration_lock().await;
    }

    // Second caller must acquire within 100ms (no other holder).
    let acquired = tokio::time::timeout(
        Duration::from_millis(100),
        svc.orchestration_lock(),
    )
    .await;
    assert!(
        acquired.is_ok(),
        "orchestration_lock should be acquirable immediately after the previous guard dropped"
    );
}

/// Sanity check: the lock exists on a freshly-constructed TimerTickService
/// without needing AppContext setup.
#[tokio::test]
async fn orchestration_lock_exists_on_minimal_service() {
    let event_publisher =
        Arc::new(InMemoryEventBus::new()) as Arc<_>;
    let timer_repo: Arc<dyn domain::TimerRepository + Send + Sync> =
        Arc::new(infra::adapters::timer::SqliteTimerRepository::new(
            ctx_minimal_db_pool().await,
        ));
    // For this sanity check we only need the method to exist and return a
    // guard. Use the AppContext path (above) for functional tests.
    // (This test is intentionally minimal; remove if it adds maintenance
    // burden.)
}

async fn ctx_minimal_db_pool() -> Arc<infra::adapters::DbPool> {
    // Helper retained for the minimal-service sanity test. If the AppContext
    // path covers everything you need, delete this fn and the test that
    // calls it before merging.
    unimplemented!("see AppContext-based tests above; remove this stub if unused")
}
```

> **Implementer note:** the third test (`orchestration_lock_exists_on_minimal_service`) is a placeholder sketch. If the AppContext-based tests above already give you confidence the method exists and works, delete the placeholder test and its helper before running Step 2 — do not leave a `todo!()` / `unimplemented!()` in the merged test file.

In `core/infra/tests/app/mod.rs`, add:

```rust
mod orchestration_lock;
```

- [ ] **Step 2: Run the test and verify it FAILS (method missing)**

Run: `cargo test -p infra --test main -- orchestration_lock --nocapture`
Expected: COMPILE ERROR — `method not found: orchestration_lock`. This is the failing-test step in TDD terms; the API does not exist yet.

- [ ] **Step 3: Implement the orchestration lock**

In `core/infra/src/adapters/timer/sqlite_service.rs`, modify the struct definition (currently around lines 15-22) to add the new field:

```rust
#[derive(Clone)]
pub struct TimerTickService {
    timer: Arc<Mutex<Timer>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Serializes mutating orchestrations (Tauri commands, tray handlers,
    /// `CountdownExpiredHandler`). Held for the entire body of a command.
    /// Without this, two concurrent orchestrations race on the DB and the
    /// tick loop — a race that is invisible on fast hardware but
    /// reproducibly freezes the app on macOS M1.
    ///
    /// Internal methods (`start_timer_tick_loop`, `stop_timer_tick_loop`,
    /// `load_state`, `save_state`) MUST NOT acquire this lock — they assume
    /// the caller holds it. Acquiring it inside those methods would
    /// re-entrantly deadlock.
    orchestration_lock: tokio::sync::Mutex<()>,
    event_publisher: EventPublisherArc,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}
```

Update `TimerTickService::new` (currently around lines 25-39) to initialize the field:

```rust
    pub fn new(
        event_publisher: EventPublisherArc,
        timer_repository: Arc<dyn TimerRepository + Send + Sync>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        let timer = Timer::idle();

        Self {
            timer: Arc::new(Mutex::new(timer)),
            cancel_handle: Arc::new(Mutex::new(None)),
            orchestration_lock: tokio::sync::Mutex::new(()),
            event_publisher,
            timer_repository,
            config_repository,
        }
    }
```

Add the accessor method immediately after `stop_timer_tick_loop` (before `is_tick_loop_alive`):

```rust
    /// Acquire the global orchestration lock. Hold the returned guard for the
    /// entire body of a mutating entry point (Tauri command, tray handler,
    /// `CountdownExpiredHandler::handle`). When the guard drops, the next
    /// waiter proceeds.
    ///
    /// Internal methods on this service do NOT call this — they assume the
    /// caller has already acquired the lock.
    pub async fn orchestration_lock(
        &self,
    ) -> tokio::sync::MutexGuard<'_, ()> {
        self.orchestration_lock.lock().await
    }
```

The `Clone` derive on `TimerTickService` remains valid: `tokio::sync::Mutex<()>` is `Clone` only if its contents are — `()` is `Clone`, so this compiles. (Confirm by running `cargo build -p infra`.)

- [ ] **Step 4: Run the tests and verify PASS**

Run: `cargo test -p infra --test main -- orchestration_lock --nocapture`
Expected: PASS (both functional tests; the placeholder test was either implemented or deleted per the note in Step 1).

- [ ] **Step 5: Run the full infra suite — no regressions**

Run: `cargo test -p infra`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add core/infra/src/adapters/timer/sqlite_service.rs \
        core/infra/tests/app/orchestration_lock.rs \
        core/infra/tests/app/mod.rs
git commit -m "feat(infra/timer): add orchestration_lock primitive

Adds a tokio::sync::Mutex<()> on TimerTickService. Mutating entry points
(Tauri commands, tray handlers, CountdownExpiredHandler) will hold this
guard for their full body to serialize against each other. Internal
methods (start/stop/load/save) do not acquire it.

This is the primitive only — no callers yet. Task 3 wires it in."
```

---

## Task 3: Apply `orchestration_lock` to every mutating entry point

**Files:**
- Modify: 7 Tauri command files under `apps/tauri-app/src/commands/timer_cmd/`
- Modify: `apps/tauri-app/src/tray.rs` (4 handlers: `menu_play_pause`, `menu_reset_phase`, `menu_skip`, `menu_reset_task`)
- Modify: `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs`

**Interfaces:**
- Consumes: `TimerTickService::orchestration_lock` from Task 2.
- Produces: no signature changes. Behavior: mutating orchestrations serialize.

**Test strategy:** No new dedicated test. The Task 2 primitive test proves the lock works; the existing `core/infra/tests/app/concurrent_command_stress.rs` stress tests prove applying the lock doesn't deadlock or regress. Verification that the fix resolves the M1 freeze is manual (Task 4).

> **Pattern for every site:** acquire the guard as the FIRST statement of the entry point, before any other `await` or use case call. Bind it to `_orchestration_lock` (leading underscore = "held for side effect, not read") so the lint is silent and the drop point is unambiguous (end of scope).

- [ ] **Step 1: Apply to Tauri commands — `skip_phase`**

In `apps/tauri-app/src/commands/timer_cmd/skip_phase.rs`, immediately after the existing

```rust
pub async fn skip_phase(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
) -> Result<Timer, String> {
```

add:

```rust
    let _orchestration_lock =
        timer_tick_service.inner().orchestration_lock().await;
```

Place it before the existing `let timer_repo_arc = ...` line. The rest of the body is unchanged.

- [ ] **Step 2: Apply the same one-liner to the remaining 6 commands**

For each of these files, add the identical line as the first statement of the async body, using that command's `timer_tick_service` (or `timer_tick_service_arc` in `switch_active_task.rs`) parameter:

- `apps/tauri-app/src/commands/timer_cmd/start_timer.rs`
- `apps/tauri-app/src/commands/timer_cmd/pause_timer.rs`
- `apps/tauri-app/src/commands/timer_cmd/resume_timer.rs`
- `apps/tauri-app/src/commands/timer_cmd/reset_timer.rs`
- `apps/tauri-app/src/commands/timer_cmd/reset_timer_phase.rs`
- `apps/tauri-app/src/commands/timer_cmd/switch_active_task.rs`

In `switch_active_task.rs`, the parameter is named `timer_tick_service_arc` — use that name. The other five use `timer_tick_service`.

- [ ] **Step 3: Apply to tray handlers**

In `apps/tauri-app/src/tray.rs`, the four mutating menu handlers each use `ctx.spawn(|ctx| async move { ... })`. As the first statement inside each `async move` block, add:

```rust
        let _orchestration_lock =
            ctx.tick_service.orchestration_lock().await;
```

Add this to:
- `menu_play_pause` (around line 523)
- `menu_reset_phase` (around line 619)
- `menu_skip` (around line 685)
- `menu_reset_task` (around line 745)

Place it before any `ctx.task_repo` / `ctx.tick_service.stop_timer_tick_loop()` / use-case call. The existing `TrayCtx::spawn` signature is unchanged.

- [ ] **Step 4: Apply to `CountdownExpiredHandler`**

In `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs`, in the `handle` method, immediately after the downcast succeeds (before `let outcome = progress_phase(...)`), add:

```rust
        // Acquire the orchestration lock so we serialize against any
        // concurrent manual command (skip / pause / etc.) or tray handler.
        // Without this, a natural expiry racing with a manual skip can
        // collide on the DB and tick loop.
        let _orchestration_lock =
            self.timer_srv.orchestration_lock().await;
```

`CountdownExpiredHandler` already holds `timer_srv: Arc<TimerTickService>`, so no new field is needed.

> **Why this is safe:** `CountdownExpired` is published by the tick-loop task itself, which has already broken out of its loop by the time it publishes (see `sqlite_service.rs` lines ~149-172). The handler runs on a fresh `tokio::spawn` from the event bus. Acquiring the orchestration lock there cannot re-enter the tick loop or deadlock against it — the only contenders are the Tauri commands and tray handlers, all of which release the lock before returning.

- [ ] **Step 5: Build the whole workspace**

Run: `cargo build`
Expected: compiles with no warnings about the new bindings.

- [ ] **Step 6: Run the existing stress tests — they must still pass**

Run: `cargo test -p infra --test main -- concurrent_command_stress --nocapture`
Expected: all 3 existing stress tests pass. These are the regression net proving the lock doesn't introduce a new deadlock. If any hang, double-check that no internal method (start/stop/load) accidentally calls `orchestration_lock`.

- [ ] **Step 7: Run the full infra suite**

Run: `cargo test -p infra`
Expected: all tests pass. Pay attention to `tick_loop_invariants` — the auto-advance path now goes through a locked `CountdownExpiredHandler`; the test's `tokio::time::sleep` windows still allow the handler to settle.

- [ ] **Step 8: Commit**

```bash
git add apps/tauri-app/src/commands/timer_cmd/skip_phase.rs \
        apps/tauri-app/src/commands/timer_cmd/start_timer.rs \
        apps/tauri-app/src/commands/timer_cmd/pause_timer.rs \
        apps/tauri-app/src/commands/timer_cmd/resume_timer.rs \
        apps/tauri-app/src/commands/timer_cmd/reset_timer.rs \
        apps/tauri-app/src/commands/timer_cmd/reset_timer_phase.rs \
        apps/tauri-app/src/commands/timer_cmd/switch_active_task.rs \
        apps/tauri-app/src/tray.rs \
        core/infra/src/adapters/timer/event_handlers/countdown_expired.rs
git commit -m "fix(tauri/timer): acquire orchestration_lock at every mutating entry point

Wire Task 2's primitive into the 7 mutating Tauri commands, 4 tray
handlers, and CountdownExpiredHandler::handle. Concurrent mutating
orchestrations now serialize instead of racing on the singleton timer
and tick loop.

This is the fix for the macOS-M1-only intermittent freeze: on slower
hardware the race window widens enough that two overlapping stop/load/
start sequences collide every time. The frontend isBusy guard hides
this for UI-only clicks but cannot prevent tray-vs-UI or skip-vs-
natural-expiry collisions."
```

---

## Task 4: Manual verification on macOS M1

**Files:** none modified.

**Why this is a task and not a footnote:** the freeze is platform-specific and timing-dependent. The Linux test suite cannot confirm the fix end-to-end. M1 is the gate.

- [ ] **Step 1: Build for macOS M1**

On the M1 machine:

```bash
cargo tauri build --target aarch64-apple-darwin
# or, for faster iteration during verification:
cargo tauri dev
```

- [ ] **Step 2: Reproduce the original freeze on the PRE-fix build (sanity)**

Check out `main` (or the commit before Task 1), build, run, and rapidly click skip ~10-20 times. Confirm the freeze reproduces. This validates that the M1 environment is still in the "reproducible" state — if you can't reproduce on the pre-fix build, the bug has shifted and the verification is inconclusive.

- [ ] **Step 3: Verify the fix on the post-fix build**

Check out the Task 3 commit (head of this plan), rebuild, run, and rapidly click skip ~50 times. Then:

- [ ] No freeze (UI stays responsive, IPC calls complete)
- [ ] Tray skip + UI pause racing does not freeze
- [ ] Letting a work phase expire naturally while clicking skip does not freeze
- [ ] After 5+ minutes of rapid activity, the tick loop is still alive and the timer is accurate

- [ ] **Step 4: Decision branch**

- **All green:** the plan is complete. Close it out; do not implement Fix B unless a different freeze appears later.
- **Freeze still reproduces:** the root cause is deeper than mutex contention. Implement Fix B (`spawn_blocking` for diesel calls) as the next increment. Capture logs (`~/.local/share/<bundle>/logs/pomotoro.log` in dev; review the last 200 lines around the wedge) before reverting — they will show whether the hang is at the DB layer (Fix B territory) or somewhere else entirely.

---

## Self-Review

**Spec coverage.** The user's request was "full refactor of what's recommended to avoid the issue" with direction toward A+C. This plan delivers exactly that:
- Fix C (drop mutex across write) → Task 1
- Fix A (re-entry guard) → Tasks 2 + 3
- Fix B (spawn_blocking) explicitly deferred, gated on Task 4 outcome

**Placeholder scan.** Task 2 Step 1 contains a placeholder test (`orchestration_lock_exists_on_minimal_service` + `ctx_minimal_db_pool`) with an explicit note to delete it before merging. No other TBDs. Every step that changes code shows the actual code.

**Type consistency.** The method `pub async fn orchestration_lock(&self) -> tokio::sync::MutexGuard<'_, ()>` is defined in Task 2 and consumed identically (`svc.orchestration_lock().await` or `ctx.tick_service.orchestration_lock().await` or `self.timer_srv.orchestration_lock().await`) across Task 3's nine call sites. The struct field is `orchestration_lock: tokio::sync::Mutex<()>`. The bind name `_orchestration_lock` is consistent across all sites.

**Risk audit.**
- *Re-entrancy:* no internal method acquires the lock; only entry points do. The `CountdownExpiredHandler` runs on a fresh `tokio::spawn` from the event bus, so it cannot be re-entered from within a command that already holds the lock.
- *Deadlock with `cancel_handle`/`timer` mutexes:* entry points acquire `orchestration_lock` first, then call internal methods that acquire `cancel_handle`/`timer`. Lock ordering is consistent across all entry points; no path acquires them in the reverse order.
- *Frontend `isBusy`:* unchanged. The lock makes concurrent backend calls serialize, but the frontend already prevents them at the UI layer. Net effect: zero change for UI-only flows; big improvement for tray/UI and natural-expiry races.
- *Read-only commands (`get_timer_state`):* intentionally not locked. Reads are cheap and don't contend meaningfully.
- *`TimerTickService::Clone` derive:* `tokio::sync::Mutex<()>` is `Clone` because `()` is, so the existing derive survives.

**Verification footprint.** Linux test suite covers: (a) the bug exists pre-Fix-C (Task 1 Step 2 hangs), (b) Fix C removes it (Task 1 Step 4 passes), (c) orchestration_lock serializes (Task 2 Step 4 passes), (d) applying the lock doesn't regress (Task 3 Steps 6-7 pass). macOS M1 covers: (e) the end-to-end freeze is gone (Task 4 Step 3). The two environments are complementary, not redundant.
