use std::time::Duration;

use domain::{
    Config, ConfigRepository, TaskCyclingBehavior, TaskRepository, TaskStatus,
    TimerConfiguration,
};
use usecases::timer::{
    StartTimerPhaseCmd, complete_timer_phase, start_timer_phase,
};

use crate::{
    AppContextBuilder, TaskBuilder,
    utils::{task::get_active_task, timer::get_timer},
};

/// Build a context with no extra fixture tasks so the round-robin selector
/// only sees the tasks each test creates explicitly.
async fn setup_minimal_ctx(name: &str) -> crate::AppContext {
    AppContextBuilder::new()
        .with_name(name)
        .build()
        .await
        .expect("Failed to build test context")
}

/// Build a Config with the given cycling behavior and auto-start flags.
fn cycling_config(
    behavior: TaskCyclingBehavior,
    auto_start_breaks: bool,
    auto_start_work_after_break: bool,
) -> Config {
    let mut config = Config::default();
    config.general.task_cycling_behavior = behavior;
    config.general.auto_start_breaks = auto_start_breaks;
    config.general.auto_start_work_after_break = auto_start_work_after_break;
    config
}

/// Apply a config to the running app via the config repository.
async fn apply_config(ctx: &crate::AppContext, config: &Config) {
    ctx.config_repo.save_config(config).await.unwrap();
}

/// Drive a task from start through one work session and its following break,
/// ending with the `BreakPhaseCompleted` event that triggers auto-cycling.
async fn complete_one_full_session(
    ctx: &crate::AppContext,
    task_id: domain::TaskId,
) {
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

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Work -> break (increments session count; may complete the task)
    complete_timer_phase(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("Failed to complete work phase");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Break -> work (publishes BreakPhaseCompleted; auto-cycle fires here)
    complete_timer_phase(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("Failed to complete break phase");

    // Let async BreakPhaseCompletedHandler settle
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
}

#[tokio::test]
async fn auto_advance_cycles_when_task_completes_with_auto_start() {
    let ctx = setup_minimal_ctx(
        "auto_advance_cycles_when_task_completes_with_auto_start",
    )
    .await;

    // Arrange: AutoAdvance + auto-start both enabled
    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, true),
    )
    .await;

    let test_config = Config {
        timer: TimerConfiguration::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
        .expect("Failed to create timer configuration"),
        ..Config::default()
    };

    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(1)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task1.clone()).await.unwrap();

    let task2 = TaskBuilder::new()
        .name("Task 2")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task2.clone()).await.unwrap();

    // Act: complete task1's single session + its break
    complete_one_full_session(&ctx, task1.id()).await;

    // Assert: active task switched to task2
    let active = get_active_task(&ctx).await;
    assert_eq!(
        active.id(),
        task2.id(),
        "AutoAdvance should switch to task2 after task1 completes"
    );

    // task1 should be completed
    let task1_final =
        ctx.task_repo.get_by_id(task1.id()).await.unwrap().unwrap();
    assert!(
        task1_final.is_completed(),
        "Task1 should be completed after its single session"
    );

    // Timer should be running on task2 (auto_start_work_after_break = true)
    let timer = get_timer(&ctx).await;
    assert!(
        timer.is_running(),
        "Timer should be running on the new task after auto-start"
    );
    assert_eq!(timer.task_id(), Some(task2.id()));

    // Notification event should have been emitted
    assert!(
        ctx.ui_simulator
            .app_handle()
            .was_event_emitted("task:auto_advanced"),
        "Should emit task:auto_advanced event after cycling"
    );
}

#[tokio::test]
async fn auto_advance_cycles_even_when_auto_start_disabled() {
    let ctx =
        setup_minimal_ctx("auto_advance_cycles_even_when_auto_start_disabled")
            .await;

    // Arrange: AutoAdvance enabled, but auto-start AFTER BREAK disabled.
    // Previously this combination silently skipped cycling because the
    // timer entered Paused state and the guard required is_running.
    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, false),
    )
    .await;

    let test_config = Config {
        timer: TimerConfiguration::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
        .expect("Failed to create timer configuration"),
        ..Config::default()
    };

    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(1)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task1.clone()).await.unwrap();

    let task2 = TaskBuilder::new()
        .name("Task 2")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task2.clone()).await.unwrap();

    // Act
    complete_one_full_session(&ctx, task1.id()).await;

    // Assert: cycling STILL happened despite auto_start_work_after_break=false
    let active = get_active_task(&ctx).await;
    assert_eq!(
        active.id(),
        task2.id(),
        "AutoAdvance must cycle to task2 even when auto_start_work_after_break is disabled"
    );

    assert!(
        ctx.ui_simulator
            .app_handle()
            .was_event_emitted("task:auto_advanced"),
        "Should emit task:auto_advanced event even without auto-start"
    );
}

#[tokio::test]
async fn auto_advance_skips_completed_tasks_round_robin() {
    let ctx =
        setup_minimal_ctx("auto_advance_skips_completed_tasks_round_robin")
            .await;

    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, true),
    )
    .await;

    let test_config = Config {
        timer: TimerConfiguration::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
        .expect("Failed to create timer configuration"),
        ..Config::default()
    };

    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(1)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task1.clone()).await.unwrap();

    // task2 is already completed — must be skipped by the round-robin selector
    let task2 = TaskBuilder::new()
        .name("Task 2")
        .max_sessions(4)
        .current_sessions(4)
        .status(TaskStatus::Completed)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task2.clone()).await.unwrap();

    let task3 = TaskBuilder::new()
        .name("Task 3")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task3.clone()).await.unwrap();

    // Act
    complete_one_full_session(&ctx, task1.id()).await;

    // Assert: cycled straight to task3, skipping the completed task2
    let active = get_active_task(&ctx).await;
    assert_eq!(
        active.id(),
        task3.id(),
        "AutoAdvance should skip completed task2 and land on task3"
    );

    assert!(
        ctx.ui_simulator
            .app_handle()
            .was_event_emitted("task:auto_advanced"),
        "Should emit task:auto_advanced event after round-robin cycling"
    );
}

#[tokio::test]
async fn auto_advance_noop_when_no_incomplete_tasks() {
    let ctx =
        setup_minimal_ctx("auto_advance_noop_when_no_incomplete_tasks").await;

    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, true),
    )
    .await;

    let test_config = Config {
        timer: TimerConfiguration::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
        .expect("Failed to create timer configuration"),
        ..Config::default()
    };

    // Only one task exists; completing it leaves nothing to cycle to.
    let task1 = TaskBuilder::new()
        .name("Only Task")
        .max_sessions(1)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task1.clone()).await.unwrap();

    // Act: should not panic or error
    complete_one_full_session(&ctx, task1.id()).await;

    // Assert: active task stays on task1 (nothing to cycle to)
    let timer = get_timer(&ctx).await;
    assert_eq!(
        timer.task_id(),
        Some(task1.id()),
        "Timer should remain on task1 when there are no other tasks to cycle to"
    );

    assert!(
        !ctx.ui_simulator
            .app_handle()
            .was_event_emitted("task:auto_advanced"),
        "Should NOT emit task:auto_advanced when there is nothing to cycle to"
    );
}

#[tokio::test]
async fn manual_mode_does_not_auto_advance() {
    let ctx = setup_minimal_ctx("manual_mode_does_not_auto_advance").await;

    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::Manual, true, true),
    )
    .await;

    let test_config = Config {
        timer: TimerConfiguration::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
        .expect("Failed to create timer configuration"),
        ..Config::default()
    };

    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(1)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task1.clone()).await.unwrap();

    let task2 = TaskBuilder::new()
        .name("Task 2")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task2.clone()).await.unwrap();

    // Act
    complete_one_full_session(&ctx, task1.id()).await;

    // Assert: in Manual mode the active task is NOT switched
    let active = get_active_task(&ctx).await;
    assert_eq!(
        active.id(),
        task1.id(),
        "Manual mode must not auto-advance even when another task is available"
    );

    assert!(
        !ctx.ui_simulator
            .app_handle()
            .was_event_emitted("task:auto_advanced"),
        "Should NOT emit task:auto_advanced in Manual mode"
    );
}

#[tokio::test]
async fn auto_advanced_event_carries_correct_payload() {
    let ctx =
        setup_minimal_ctx("auto_advanced_event_carries_correct_payload").await;

    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, true),
    )
    .await;

    let test_config = Config {
        timer: TimerConfiguration::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
        .expect("Failed to create timer configuration"),
        ..Config::default()
    };

    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(1)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task1.clone()).await.unwrap();

    let task2 = TaskBuilder::new()
        .name("Task 2")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task2.clone()).await.unwrap();

    complete_one_full_session(&ctx, task1.id()).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type("task:auto_advanced");
    assert_eq!(events.len(), 1, "Exactly one auto_advanced event expected");

    let payload = &events[0].payload;
    assert_eq!(
        payload["from_task_id"],
        serde_json::json!(task1.id()),
        "Payload from_task_id should match the completed task"
    );
    assert_eq!(
        payload["to_task_id"],
        serde_json::json!(task2.id()),
        "Payload to_task_id should match the next task"
    );
}
