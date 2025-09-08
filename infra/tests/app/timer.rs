use std::{any::TypeId, time::Duration};

use crate::{
    AppContextBuilder,
    utils::{
        self, assert_utils,
        setup::{setup_ctx, setup_ctx_with_timer},
        timer::{get_timer, get_timer_state},
    },
};
use domain::{
    Phase, TaskRepository, TaskStatus, TimerPaused, TimerRepository,
    TimerReset, TimerStarted, TimerState, TimerStatus, event_names,
    shared_kernel::events::AppStarted,
};
use usecases::{
    CreateTaskCmd, SwitchTaskCmd, create_task, timer::complete_work_session,
};
use usecases::{
    switch_task,
    timer::{
        StartTimerSessionCmd, pause_timer_session, reset_timer_session,
        start_timer_session,
    },
};

#[tokio::test]
async fn timer_should_initialize_in_idle_state() {
    let ctx = setup_ctx("timer_should_initialize_in_idle_state").await;

    let timer_state = get_timer_state(&ctx).await;

    assert_eq!(timer_state.status(), TimerStatus::Stopped);
    assert_eq!(timer_state, TimerState::Idle);

    assert_utils::assert_event_published(&ctx, TypeId::of::<AppStarted>());
}

#[tokio::test]
async fn timer_should_start_from_idle_state() {
    // arrange
    let ctx = setup_ctx("timer_should_start_from_idle_state").await;

    let timer = get_timer(&ctx).await;
    let task_id = timer.active_task_id().expect("Task id should be set");
    let timer_idle_state = timer.state().clone();
    // act
    let result = start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task_id),
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // assert
    assert!(result.is_ok());

    assert_eq!(timer_idle_state, TimerState::Idle);

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::STATUS_CHANGED,
    );

    assert_utils::assert_event_published(&ctx, TypeId::of::<TimerStarted>());

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
    let task_id = timer.active_task_id().expect("Task id should be set");

    let result = start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task_id),
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert_utils::assert_event_published(&ctx, TypeId::of::<TimerStarted>());

    assert!(result.is_err());
}

#[tokio::test]
async fn timer_should_prevent_task_switch_while_timer_is_running() {
    let ctx = setup_ctx_with_timer(
        "timer_should_prevent_task_switch_while_timer_is_running",
    )
    .await;

    let timer = get_timer(&ctx).await;
    let task_id = timer.active_task_id().expect("Task id should be set");

    let result = switch_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.task_cycling_service.clone(),
        ctx.event_bus.clone(),
        SwitchTaskCmd {
            task_id: task_id.as_str(),
        },
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn timer_should_pause_when_running() {
    // Arrange
    let ctx = setup_ctx_with_timer("timer_should_pause_when_running").await;

    let timer = get_timer(&ctx).await;

    let task_id = timer.active_task_id().expect("Task id should be set");

    let task_config = utils::task::get_active_task(&ctx).await.config;

    let task_config = task_config
        .timer
        .with_work_duration(Duration::from_secs(60))
        .expect("Failed to create timer configuration");

    timer.remaining_seconds(Some(&task_config));

    // Act
    let result = pause_timer_session(
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
    assert_eq!(timer.active_task_id(), Some(task_id));

    assert_eq!(timer_after_pause.is_paused(), true);

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::STATUS_CHANGED,
    );

    assert_utils::assert_event_published(&ctx, TypeId::of::<TimerPaused>());
}

#[tokio::test]
async fn timer_should_reset_to_initial_state() {
    let ctx = setup_ctx_with_timer("timer_should_reset_to_initial_state").await;

    let timer = get_timer(&ctx).await;
    let task_id = timer.active_task_id().expect("Task id should be set");

    let result = reset_timer_session(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let timer_after_reset = get_timer(&ctx).await;

    assert_eq!(result.is_ok(), true);
    assert_eq!(timer_after_reset.is_idle(), true);
    assert_eq!(timer_after_reset.active_task_id(), Some(task_id));

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::RESET,
    );

    assert_utils::assert_event_published(&ctx, TypeId::of::<TimerReset>());
}

#[tokio::test]
async fn timer_should_start_with_specific_task() {
    let ctx = setup_ctx("should_start_timer_with_specific_task").await;

    let timer = get_timer(&ctx).await;
    let task_id = timer.active_task_id().expect("Task id should be set");

    let result = start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task_id),
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let timer = get_timer(&ctx).await;

    assert!(result.is_ok());
    assert_eq!(timer.active_task_id(), Some(task_id));
    assert_eq!(timer.state().status(), TimerStatus::Running);
    assert_eq!(timer.state().is_work_phase(), true);
    assert_eq!(timer.state().remaining_seconds(), 1500);

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::timer::STATUS_CHANGED,
    );

    assert_utils::assert_event_published(&ctx, TypeId::of::<TimerStarted>());
}

#[tokio::test]
async fn completing_work_session_should_increment_task_counter() {
    let ctx = setup_ctx_with_timer(
        "completing_work_session_should_increment_task_counter",
    )
    .await;

    let old_timer = get_timer(&ctx).await;

    let old_task = utils::task::get_active_task(&ctx).await;

    let result =
        complete_work_session(ctx.task_repo.clone(), ctx.timer_repo.clone())
            .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let task = utils::task::get_active_task(&ctx).await;
    let new_timer = get_timer(&ctx).await;

    assert!(result.is_ok());

    assert_eq!(old_timer.state().is_work_phase(), true);
    assert_eq!(new_timer.state().is_break_phase(), true);

    assert_eq!(old_task.current_sessions, 0);
    assert_eq!(task.current_sessions, 1);

    // let new_timer = get_timer(&ctx).await;
    // assert_eq!(old_timer.active_task_id(), new_timer.active_task_id());
    // assert_eq!(old_timer.state().is_work_phase(), true);

    // assert_eq!(new_timer.state().is_break_phase(), true);
}
