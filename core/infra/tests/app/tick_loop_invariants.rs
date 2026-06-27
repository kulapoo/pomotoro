//! Regression tests for the tick-loop ownership invariant.
//!
//! After the architectural change in this plan, domain event handlers MUST NOT
//! mutate `TimerTickService`'s `cancel_handle`. They are pure UI emitters. The
//! original auto-advance bug was caused by `TimerResetHandler` and
//! `TimerStartedHandler` racing on `cancel_handle` via detached `tokio::spawn`
//! tasks in `InMemoryEventBus::publish`. These tests lock the invariant.

use std::time::Duration;

use domain::{
    Config, ConfigRepository, EventPublisher, Phase, TaskCyclingBehavior,
    TaskRepository, timer::events::CountdownExpired,
};
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

/// Timer-driven auto-advance: when a break-phase countdown expires for a
/// completed task with `AutoAdvance` cycling and `auto_start_work_after_break`
/// enabled, `CountdownExpiredHandler` must leave the tick loop ALIVE on the
/// new task.
///
/// Before the fix, `TimerResetHandler` (spawned by `reset_timer_to_idle`
/// inside `progress_phase`) could abort the loop that `CountdownExpiredHandler`
/// had just started. Running the scenario repeatedly catches the race if it
/// ever regresses.
///
/// Both phase transitions are driven through the real event path
/// (`CountdownExpired` → `CountdownExpiredHandler` → `progress_phase`) so the
/// test exercises the same orchestration that runs in production.
#[tokio::test]
async fn auto_advance_leaves_tick_loop_alive_on_new_task() {
    for iteration in 0..20 {
        let ctx = AppContextBuilder::new()
            .with_name(format!("auto_advance_tick_loop_alive_{iteration}"))
            .build()
            .await
            .expect("Failed to build test context");

        // Enable auto-cycling and auto-start of the next work phase.
        // `should_auto_cycle` requires `TaskCyclingBehavior::AutoAdvance`.
        let mut config = Config::default();
        config.general.task_cycling_behavior = TaskCyclingBehavior::AutoAdvance;
        config.general.auto_start_work_after_break = true;
        ctx.config_repo.save_config(&config).await.unwrap();

        // Two tasks so cycling has somewhere to go. task1 is one session away
        // from completion; task2 is the cycle target.
        let task1 = TaskBuilder::new()
            .name("Task 1")
            .max_sessions(1)
            .config(config.clone())
            .build();
        let task1_id = task1.id();
        let task2 = TaskBuilder::new()
            .name("Task 2")
            .max_sessions(4)
            .config(config.clone())
            .build();
        let task2_id = task2.id();
        ctx.task_repo.create(task1).await.unwrap();
        ctx.task_repo.create(task2).await.unwrap();

        // Start task1's work phase.
        start_timer_phase(
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_bus.clone(),
            StartTimerPhaseCmd {
                task_id: Some(task1_id),
            },
        )
        .await
        .expect("start work phase");

        // Work-phase countdown expiry: the handler runs `progress_phase(Work)`,
        // which increments task1's session count to its `max_sessions` (1) —
        // marking task1 completed — and, with `auto_start_breaks` at its
        // default, auto-starts the ShortBreak phase (starting a break tick
        // loop). This puts the system in the cycle branch's precondition
        // state: task1 completed, timer on a break phase.
        ctx.event_bus
            .publish(Box::new(CountdownExpired::new(Phase::Work, task1_id)));

        // Let the handler's spawned orchestration settle.
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Reset the loop handle so the only loop alive after the next event is
        // the one `CountdownExpiredHandler` starts on the cycled task.
        ctx.timer_tick_service.stop_timer_tick_loop().await.unwrap();

        // Break-phase countdown expiry for the completed task: the handler
        // runs `progress_phase(ShortBreak)`, which hits the cycle branch
        // (`task.is_completed() && from_phase ∈ {Short,Long}Break &&
        // should_auto_cycle`). It selects task2 round-robin, switches the
        // active task, resets the timer, and auto-starts task2's Work phase.
        // The handler must leave the tick loop ALIVE on task2.
        ctx.event_bus.publish(Box::new(CountdownExpired::new(
            Phase::ShortBreak,
            task1_id,
        )));

        // Give the spawned orchestration time to finish (cycle + start loop).
        tokio::time::sleep(Duration::from_millis(500)).await;

        assert!(
            ctx.timer_tick_service.is_tick_loop_alive().await,
            "iteration {iteration}: auto-advance left the tick loop dead"
        );

        let bound_task =
            ctx.timer_tick_service.get_current_timer().await.task_id();
        assert_eq!(
            bound_task,
            Some(task2_id),
            "iteration {iteration}: tick loop is not bound to the cycled task"
        );
    }
}
