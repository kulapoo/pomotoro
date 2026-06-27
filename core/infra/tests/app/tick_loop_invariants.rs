//! Regression tests for the tick-loop ownership invariant.
//!
//! After the architectural change in this plan, domain event handlers MUST NOT
//! mutate `TimerTickService`'s `cancel_handle`. They are pure UI emitters. The
//! original auto-advance bug was caused by `TimerResetHandler` and
//! `TimerStartedHandler` racing on `cancel_handle` via detached `tokio::spawn`
//! tasks in `InMemoryEventBus::publish`. These tests lock the invariant.

use std::time::Duration;

use domain::{Config, EventPublisher, TaskRepository};
use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

use crate::{AppContextBuilder, TaskBuilder};

/// Publishing `TimerReset`, `TimerPaused`, `TimerStarted`, or `TaskReset` must
/// NOT stop (or start) an already-running tick loop. Before the fix,
/// `TimerResetHandler` / `TimerPausedHandler` / `TaskResetHandler` aborted the
/// handle via `stop_timer_tick_loop`, and `TimerStartedHandler` raced with
/// them on `start_timer_tick_loop`.
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
    let timer_config = task.config().timer.clone();
    ctx.task_repo.create(task).await.unwrap();

    // Start the timer via the normal orchestrator entry point. It publishes
    // a `TimerStarted` event; on master, `TimerStartedHandler` reacts by
    // starting the tick loop on a detached task.
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
    ctx.timer_tick_service
        .start_timer_tick_loop(Some(timer_config.clone()), None)
        .await
        .expect("Failed to start tick loop");

    // Let any spawned handler from start_timer_phase settle.
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(
        ctx.timer_tick_service.is_tick_loop_alive().await,
        "precondition: tick loop should be alive before publishing events"
    );

    // Snapshot the values needed to construct the domain events. The actual
    // shapes (verified in core/domain/src/{timer,task}/events/) use `::new`
    // constructors; the field set differs from the brief's guesses.
    let (phase, remaining_seconds) = ctx
        .timer_tick_service
        .with_timer(|t| (t.state().phase(), t.state().remaining_seconds()))
        .await;

    let reset_event = Box::new(domain::TimerReset::new(
        task_id,
        phase,
        1,
        timer_config.clone(),
    ));
    let paused_event = Box::new(domain::TimerPaused::new(
        task_id,
        phase,
        remaining_seconds,
        1,
        timer_config.clone(),
    ));
    let started_event = Box::new(domain::TimerStarted::new(
        task_id,
        phase,
        timer_config.work_duration.as_secs() as u32,
        1,
    ));
    let task_reset_event =
        Box::new(domain::TaskReset::new(task_id, None, None, None, None, 1));

    ctx.event_bus.publish(reset_event);
    ctx.event_bus.publish(paused_event);
    ctx.event_bus.publish(started_event);
    ctx.event_bus.publish(task_reset_event);

    // Give the spawned handlers time to (incorrectly) mutate the handle.
    tokio::time::sleep(Duration::from_millis(500)).await;

    assert!(
        ctx.timer_tick_service.is_tick_loop_alive().await,
        "INVARIANT VIOLATED: a domain event handler mutated cancel_handle. \
         Handlers must be UI-only emitters; orchestrators own the tick loop."
    );
}
