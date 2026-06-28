//! Stress tests for concurrent orchestrator access to `TimerTickService`.
//!
//! These tests exist to reproduce an intermittent UI freeze reported when the
//! user rapidly presses "skip phase" in the Tauri app. The frontend `isBusy`
//! guard coalesces rapid UI clicks, but the backend has no command-level
//! serialization. If a manual skip ever overlaps with another path that
//! touches the tick service (a natural `CountdownExpired` from the running
//! loop, an event-driven `fetchTimer`, or simply a queued click that slipped
//! through), both run their full stop/load/start sequence against shared
//! state and can wedge the runtime.
//!
//! Reproduction strategy: run the same stop/load/start sequence the
//! `skip_phase` Tauri command runs, from many tasks at once, on a
//! multi-threaded tokio runtime. If the sequence is not safe under
//! concurrency, the test will hang and be killed by the timeout.

use std::sync::Arc;
use std::time::Duration;

use domain::{
    Config, ConfigRepository, EventPublisher, TaskId, TaskRepository,
    TimerRepository,
};
use infra::adapters::TimerTickService;
use usecases::timer::{
    StartTimerPhaseCmd, skip_timer_phase, start_timer_phase,
};

use crate::{AppContextBuilder, TaskBuilder};

/// Bundle of shared state passed into each spawned orchestrator. Cloning
/// the Arcs out of `AppContext` once up front sidesteps the lifetime issues
/// of borrowing `&ctx` across `tokio::spawn`.
#[derive(Clone)]
struct SkipCmdDeps {
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    timer_tick_service: Arc<TimerTickService>,
}

impl SkipCmdDeps {
    fn from_ctx(ctx: &crate::core::context::AppContext) -> Self {
        Self {
            task_repo: ctx.task_repo.clone(),
            timer_repo: ctx.timer_repo.clone(),
            event_publisher: ctx.event_bus.clone(),
            timer_tick_service: ctx.timer_tick_service.clone(),
        }
    }
}

/// Mirror of the `skip_phase` Tauri command body (`apps/tauri-app/src/commands/
/// timer_cmd/skip_phase.rs`). Each spawned task runs the full sequence a real
/// command invocation runs, against the same shared `TimerTickService`.
async fn mimic_skip_phase_command(
    deps: SkipCmdDeps,
    task_id: TaskId,
    timer_cfg: domain::TimerConfiguration,
) -> Result<(), String> {
    skip_timer_phase(
        deps.task_repo.clone(),
        deps.timer_repo.clone(),
        deps.event_publisher.clone(),
        task_id,
    )
    .await
    .map_err(|e| format!("skip_timer_phase: {e}"))?;

    deps.timer_tick_service
        .stop_timer_tick_loop()
        .await
        .map_err(|e| format!("stop: {e}"))?;
    deps.timer_tick_service
        .load_state()
        .await
        .map_err(|e| format!("load: {e}"))?;

    let updated = deps
        .timer_repo
        .get()
        .await
        .map_err(|e| format!("get: {e}"))?;
    if updated.is_running() {
        deps.timer_tick_service
            .start_timer_tick_loop(Some(timer_cfg))
            .await
            .map_err(|e| format!("start: {e}"))?;
    }
    Ok(())
}

/// Reproduce the freeze: many concurrent skip-phase orchestrations racing
/// with a live tick loop must all complete within a bounded time.
///
/// On master this is expected to either deadlock (timer mutex held across
/// DB write while another caller queues on `cancel_handle` / `timer`) or
/// starve the tokio runtime via blocking diesel calls. Either failure mode
/// will exceed the 30s timeout.
///
/// Loops 50 times because the production bug is intermittent — a single
/// iteration is unlikely to hit the race window.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_skip_phase_does_not_deadlock() {
    for iter in 0..50 {
        let ctx = AppContextBuilder::new()
            .with_name(format!("concurrent_skip_phase_iter_{iter}"))
            .build()
            .await
            .expect("Failed to build test context");

        // Long-lived task so repeated skips have somewhere to go.
        let task = TaskBuilder::new()
            .name("Stress task")
            .max_sessions(255)
            .config(Config::default())
            .build();
        let task_id = task.id();
        let timer_cfg = task.config().timer.clone();
        ctx.task_repo.create(task).await.unwrap();

        // Start the timer + tick loop the way a real Tauri command would.
        start_timer_phase(
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_bus.clone(),
            StartTimerPhaseCmd {
                task_id: Some(task_id),
            },
        )
        .await
        .expect("start_timer_phase");
        ctx.timer_tick_service
            .start_timer_tick_loop(Some(timer_cfg.clone()))
            .await
            .expect("start tick loop");

        // Let the loop spin up so its tick task is competing for the timer mutex.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let deps = SkipCmdDeps::from_ctx(&ctx);
        let outcome = tokio::time::timeout(Duration::from_secs(30), async {
            let mut handles = Vec::new();
            // 8 concurrent orchestrations — well within the r2d2 pool
            // (max_size 10) and the event-bus semaphore (100 permits),
            // so any hang must come from a real deadlock/starvation, not
            // resource exhaustion.
            for i in 0..8 {
                let deps = deps.clone();
                let cfg = timer_cfg.clone();
                handles.push(tokio::spawn(async move {
                    mimic_skip_phase_command(deps, task_id, cfg)
                        .await
                        .unwrap_or_else(|e| {
                            panic!("iter {iter} task {i} failed: {e}")
                        });
                }));
            }
            for (i, h) in handles.into_iter().enumerate() {
                h.await.unwrap_or_else(|e| {
                    panic!("iter {iter} task {i} join failed: {e}")
                });
            }
        })
        .await;

        assert!(
            outcome.is_ok(),
            "DEADLOCK REPRODUCED at iteration {iter}: 8 concurrent skip_phase \
             orchestrations did not complete within 30 seconds."
        );
        // Drain any remaining tick loops so the next iteration starts clean.
        let _ = ctx.timer_tick_service.stop_timer_tick_loop().await;
    }
}

/// Race condition test: a manual `skip_phase` racing with the natural
/// `CountdownExpired` event the running tick loop fires when its countdown
/// hits zero. Both paths run the full stop/load/start sequence against the
/// shared `TimerTickService`. This is the most likely real-world trigger for
/// the intermittent freeze — the user clicks skip a fraction of a second
/// before the timer would have expired naturally.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn manual_skip_racing_with_natural_expiry_does_not_deadlock() {
    use domain::timer::events::CountdownExpired;
    use domain::{Phase, TaskCyclingBehavior};

    for iter in 0..30 {
        let mut config = Config::default();
        // Make sure auto-start is on so progress_phase exercises the start
        // branch — that's where it would clash with the concurrent manual
        // skip's own start.
        config.general.auto_start_breaks = true;
        config.general.auto_start_work_after_break = true;
        config.general.task_cycling_behavior = TaskCyclingBehavior::AutoAdvance;

        let ctx = AppContextBuilder::new()
            .with_name(format!("manual_skip_vs_expiry_iter_{iter}"))
            .build()
            .await
            .expect("Failed to build test context");
        ctx.config_repo.save_config(&config).await.unwrap();

        let task = TaskBuilder::new()
            .name("Race task")
            .max_sessions(255)
            .config(config.clone())
            .build();
        let task_id = task.id();
        let timer_cfg = task.config().timer.clone();
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
        .expect("start_timer_phase");
        ctx.timer_tick_service
            .start_timer_tick_loop(Some(timer_cfg.clone()))
            .await
            .expect("start tick loop");

        tokio::time::sleep(Duration::from_millis(50)).await;

        let deps = SkipCmdDeps::from_ctx(&ctx);
        let event_bus = ctx.event_bus.clone();

        let outcome = tokio::time::timeout(Duration::from_secs(30), async {
            // Fire manual skip + CountdownExpired simultaneously, plus
            // a second skip shortly after — three racing orchestrations.
            let skip1 = tokio::spawn(mimic_skip_phase_command(
                deps.clone(),
                task_id,
                timer_cfg.clone(),
            ));
            let expiry = tokio::spawn({
                let event_bus = event_bus.clone();
                async move {
                    // The event bus spawns the handler; the handler runs
                    // progress_phase which itself does stop/load/start.
                    event_bus.publish(Box::new(CountdownExpired::new(
                        Phase::Work,
                        task_id,
                    )));
                }
            });
            // Tiny stagger to simulate a second user click 50ms later.
            tokio::time::sleep(Duration::from_millis(50)).await;
            let skip2 = tokio::spawn(mimic_skip_phase_command(
                deps.clone(),
                task_id,
                timer_cfg.clone(),
            ));

            skip1
                .await
                .unwrap_or_else(|e| panic!("iter {iter} skip1: {e}"))
                .expect("iter {iter} skip1 failed");
            expiry
                .await
                .unwrap_or_else(|e| panic!("iter {iter} expiry: {e}"));
            skip2
                .await
                .unwrap_or_else(|e| panic!("iter {iter} skip2: {e}"))
                .expect("iter {iter} skip2 failed");
            // Give any spawned event-bus handlers time to settle.
            tokio::time::sleep(Duration::from_millis(200)).await;
        })
        .await;

        assert!(
            outcome.is_ok(),
            "DEADLOCK REPRODUCED at iteration {iter}: manual skip + natural \
             expiry + second skip did not settle within 30 seconds."
        );
        let _ = ctx.timer_tick_service.stop_timer_tick_loop().await;
    }
}

/// Variant: rapid sequential skips (single-threaded, no overlap) followed by
/// a check that the tick loop is still responsive. This mirrors the literal
/// user report — "rapidly press skip → freeze" — without depending on
/// multi-thread scheduling. Acts as a regression net for state corruption
/// that leaves the loop dead even when no true concurrency occurred.
#[tokio::test(flavor = "current_thread")]
async fn rapid_sequential_skips_leave_loop_responsive() {
    let ctx = AppContextBuilder::new()
        .with_name("rapid_sequential_skips_leave_loop_responsive")
        .build()
        .await
        .expect("Failed to build test context");

    let task = TaskBuilder::new()
        .name("Rapid task")
        .max_sessions(50)
        .config(Config::default())
        .build();
    let task_id = task.id();
    let timer_cfg = task.config().timer.clone();
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
    .expect("start_timer_phase");
    ctx.timer_tick_service
        .start_timer_tick_loop(Some(timer_cfg.clone()))
        .await
        .expect("start tick loop");

    tokio::time::sleep(Duration::from_millis(200)).await;

    let deps = SkipCmdDeps::from_ctx(&ctx);
    let outcome = tokio::time::timeout(Duration::from_secs(20), async {
        for _ in 0..15 {
            mimic_skip_phase_command(deps.clone(), task_id, timer_cfg.clone())
                .await
                .expect("skip phase");
        }
    })
    .await;
    assert!(
        outcome.is_ok(),
        "Sequential rapid skips did not complete within 20 seconds — \
         single-threaded variant of the freeze."
    );
}
