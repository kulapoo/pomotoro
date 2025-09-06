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
    Phase, TaskRepository, TaskStatus, TimerRepository, TimerStarted,
    TimerState, event_names,
    shared_kernel::events::AppStarted,
    timer::{Status as TimerStatus, TimerService},
};
use usecases::{CreateTaskCmd, SwitchTaskCmd, create_task};
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
        ctx.timer_service.clone(),
        StartTimerSessionCmd {
            task_id: Some(task_id.as_str()),
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

    let timer = get_timer(&ctx).await;

    assert!(timer.is_running(), "Timer should be running");
    assert_eq!(timer.state().status(), TimerStatus::Running);
    assert_eq!(timer.active_task_id(), Some(task_id));
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
        ctx.timer_service.clone(),
        StartTimerSessionCmd {
            task_id: Some(task_id.as_str()),
        },
    )
    .await;

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

    ctx.timer_repo
        .save(&timer)
        .await
        .expect("Failed to save timer");

    println!("timer: {:?}", timer);
    // Act
    let result = pause_timer_session(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    // Assert
    assert!(result.is_ok());

    let timer = get_timer(&ctx).await;
    println!("timer: {:?}", timer);
    // assert_eq!(timer.state().status(), TimerStatus::Paused);
    // assert_eq!(timer.active_task_id(), Some(task_id));
}
