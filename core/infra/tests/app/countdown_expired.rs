use std::time::Duration;

use domain::{
    Config, ConfigRepository, EventPublisher, Phase, TaskRepository,
    event_names::ui_listeners, timer::events::CountdownExpired,
};
use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

use crate::{AppContextBuilder, TaskBuilder, utils::assert_utils};

/// When `auto_start_breaks` is disabled, a work-phase countdown expiry
/// produces a `Paused` outcome. The handler must emit BOTH a timer signal
/// (`status_changed`) and a task signal (`progress_updated`) so the UI can
/// refresh the countdown and the session dots.
#[tokio::test]
async fn paused_outcome_emits_status_and_progress() {
    let ctx = AppContextBuilder::new()
        .with_name("paused_outcome_emits_status_and_progress")
        .build()
        .await
        .expect("Failed to build test context");

    let mut config = Config::default();
    config.general.auto_start_breaks = false;
    ctx.config_repo.save_config(&config).await.unwrap();

    let task = TaskBuilder::new()
        .name("Test Task")
        .max_sessions(4)
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
    .expect("Failed to start work phase");

    tokio::time::sleep(Duration::from_millis(100)).await;

    ctx.ui_simulator.app_handle().clear_events();

    ctx.event_bus
        .publish(Box::new(CountdownExpired::new(Phase::Work, task_id)));

    tokio::time::sleep(Duration::from_millis(200)).await;

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        ui_listeners::timer::STATUS_CHANGED,
    );
    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        ui_listeners::task::PROGRESS_UPDATED,
    );

    let task_after = ctx.task_repo.get_by_id(task_id).await.unwrap().unwrap();
    assert_eq!(task_after.current_sessions(), 1);
}
