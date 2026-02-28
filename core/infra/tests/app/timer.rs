use std::any::TypeId;

use crate::utils::{
    self, assert_utils,
    setup::{setup_ctx, setup_ctx_with_existing_db, setup_ctx_with_timer},
    timer::{get_timer, get_timer_state},
};
use domain::{
    PhaseSkipped, TaskRepository, TimerPaused, TimerReset, TimerStarted,
    TimerState, TimerStatus, event_names, shared_kernel::events::AppStarted,
};
use usecases::{
    SwitchTaskCmd, timer::complete_timer_phase, timer::skip_timer_phase,
};
use usecases::{
    switch_task,
    timer::{
        StartTimerPhaseCmd, pause_timer_phase, reset_timer_phase,
        start_timer_phase,
    },
};

#[tokio::test]
async fn timer_should_initialize_in_idle_state() {
    let ctx = setup_ctx("timer_should_initialize_in_idle_state").await;

    let timer_state = get_timer_state(&ctx).await;

    assert_eq!(timer_state.status(), TimerStatus::Stopped);
    assert_eq!(timer_state, TimerState::Idle);

    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<AppStarted>());
}

#[tokio::test]
async fn timer_should_start_from_idle_state() {
    // arrange
    let ctx = setup_ctx("timer_should_start_from_idle_state").await;

    let task = ctx
        .task_repo
        .get_default_task()
        .await
        .expect("Task should be created");

    let task_id = task.expect("Task should be created").id;

    let timer_srv = ctx.timer_tick_service.clone();

    let timer_idle_state = timer_srv.get_current_timer().await.state().clone();

    // Timer is already associated with task_id at construction time
    // No need to set_active_task anymore

    let result = start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd {
            task_id: Some(task_id),
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // // assert
    assert!(result.is_ok());

    assert_eq!(timer_idle_state, TimerState::Idle);

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::STATUS_CHANGED,
    );

    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TimerStarted>());

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let timer = get_timer(&ctx).await;
    assert!(timer.is_running(), "Timer should be running");
}

#[tokio::test]
async fn timer_should_not_start_when_already_running() {
    let ctx =
        setup_ctx_with_timer("timer_should_not_start_when_already_running")
            .await;

    let timer = get_timer(&ctx).await;
    let task_id = timer.task_id();

    let result = start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd {
            task_id: Some(task_id),
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TimerStarted>());

    assert!(result.is_err());
}

#[tokio::test]
async fn timer_should_prevent_task_switch_while_timer_is_running() {
    let ctx = setup_ctx_with_timer(
        "timer_should_prevent_task_switch_while_timer_is_running",
    )
    .await;

    let timer = get_timer(&ctx).await;
    let task_id = timer.task_id();

    let result = switch_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        SwitchTaskCmd { task_id },
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn timer_should_pause_when_running() {
    // Arrange
    let ctx = setup_ctx_with_timer("timer_should_pause_when_running").await;

    let timer = get_timer(&ctx).await;

    let task_id = timer.task_id();

    // Act
    let result = pause_timer_phase(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let timer_after_pause = get_timer(&ctx).await;

    // Assert
    assert_eq!(result.is_ok(), true);

    assert_eq!(timer.is_running(), true);
    assert_eq!(timer.task_id(), task_id);

    assert_eq!(timer_after_pause.is_paused(), true);

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::STATUS_CHANGED,
    );

    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TimerPaused>());
}

#[tokio::test]
async fn timer_should_reset_to_initial_state() {
    let ctx = setup_ctx_with_timer("timer_should_reset_to_initial_state").await;

    let timer = get_timer(&ctx).await;
    let task_id = timer.task_id();

    let result = reset_timer_phase(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let timer_after_reset = get_timer(&ctx).await;

    assert_eq!(result.is_ok(), true);
    assert_eq!(timer_after_reset.remaining_seconds(None), 1500);
    assert_eq!(timer_after_reset.task_id(), task_id);

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::RESET,
    );

    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TimerReset>());
}

#[tokio::test]
async fn timer_should_start_with_specific_task() {
    let ctx =
        setup_ctx_with_timer("should_start_timer_with_specific_task").await;

    let timer = get_timer(&ctx).await;
    let task_id = timer.task_id();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let timer = get_timer(&ctx).await;

    assert_eq!(timer.task_id(), task_id);
    assert_eq!(timer.state().status(), TimerStatus::Running);
    assert_eq!(timer.state().is_work_phase(), true);
    assert_eq!(timer.state().remaining_seconds(), 1500);

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::STATUS_CHANGED,
    );

    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TimerStarted>());
}

#[tokio::test]
async fn timer_should_complete_phase() {
    let ctx = setup_ctx_with_timer("timer_should_complete_phase").await;

    let old_timer = get_timer(&ctx).await;

    let old_task = utils::task::get_active_task(&ctx).await;

    let complete_work_phase_result = complete_timer_phase(
        old_task.id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let task = utils::task::get_active_task(&ctx).await;
    let new_timer = get_timer(&ctx).await;

    assert!(complete_work_phase_result.is_ok());

    assert_eq!(old_timer.state().is_work_phase(), true);
    assert_eq!(new_timer.state().is_break_phase(), true);

    assert_eq!(old_task.current_sessions, 0);
    assert_eq!(task.current_sessions, 1);
}

#[tokio::test]
async fn timer_should_skip_phase() {
    let ctx = setup_ctx_with_timer("timer_should_skip_phase").await;

    let old_timer = get_timer(&ctx).await;
    let old_task = utils::task::get_active_task(&ctx).await;

    let skip_timer_result = skip_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        old_task.id,
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let new_timer = get_timer(&ctx).await;
    let new_task = utils::task::get_active_task(&ctx).await;

    assert!(skip_timer_result.is_ok());

    assert_eq!(old_timer.state().is_work_phase(), true);
    assert_eq!(new_timer.state().is_break_phase(), true);

    assert_eq!(old_task.current_sessions, 0);
    assert_eq!(new_task.current_sessions, 1);

    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<PhaseSkipped>());
}

#[tokio::test]
async fn timer_should_decrement_timer_counter() {
    let ctx =
        setup_ctx_with_timer("timer_should_increment_timer_counter").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    let old_task = utils::task::get_active_task(&ctx).await;

    let pause_timer_result = pause_timer_phase(
        old_task.id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    ctx.timer_tick_service
        .load_state()
        .await
        .expect("Failed to load timer");

    let new_timer = ctx.timer_tick_service.get_current_timer().await;

    assert!(pause_timer_result.is_ok());

    let pause_from = new_timer.pause_from().expect("Pause from should be set");

    let pause_from_remaining_seconds = pause_from.remaining_seconds();

    assert_eq!(
        new_timer.state().remaining_seconds(),
        pause_from_remaining_seconds
    );
}

#[tokio::test]
async fn timer_state_should_persist_across_restarts() {
    let ctx =
        setup_ctx_with_timer("timer_state_should_persist_across_restarts")
            .await;

    ctx.timer_tick_service
        .update_timer(|timer| {
            timer.set_remaining_seconds(60 * 10);
            Ok(())
        })
        .await
        .expect("Failed to update timer");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let ctx = setup_ctx_with_existing_db(ctx.db).await;

    let timer = get_timer(&ctx).await;

    assert_eq!(timer.state().remaining_seconds(), 600);

    assert_eq!(timer.state().is_running(), true);
}

#[tokio::test]
async fn timer_should_publish_events_on_all_state_changes() {
    let ctx = setup_ctx_with_timer(
        "timer_should_publish_events_on_all_state_changes",
    )
    .await;

    let timer = get_timer(&ctx).await;

    // Pause the timer
    let _ = pause_timer_phase(
        timer.task_id(),
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Start the timer again
    let _ = start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd {
            task_id: Some(timer.task_id()),
        },
    )
    .await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Reset the timer
    let _ = reset_timer_phase(
        timer.task_id(),
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Start the timer again
    let _ = start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd {
            task_id: Some(timer.task_id()),
        },
    )
    .await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::STATUS_CHANGED,
    );
    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::PAUSE,
    );
    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::RESET,
    );
}

#[tokio::test]
async fn task_queue_should_return_next_incomplete_task() {
    let ctx =
        setup_ctx_with_timer("timer_should_start_with_default_task").await;

    // Get all tasks and filter for incomplete ones
    let all_tasks = ctx
        .task_repo
        .get_all()
        .await
        .expect("Failed to get all tasks");

    let task_queue: Vec<_> = all_tasks
        .iter()
        .filter(|t| !t.status.is_completed())
        .cloned()
        .collect();

    assert_eq!(task_queue.len(), 1);
    assert_eq!(
        task_queue[0].id,
        ctx.task_repo
            .get_default_task()
            .await
            .unwrap()
            .expect("Task should be set")
            .id
    );
}
