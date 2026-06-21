use std::time::Duration;

use domain::{
    Config, ConfigRepository, TaskCyclingBehavior, TaskRepository, TaskStatus,
    TimerConfiguration,
};
use usecases::timer::{
    PhaseOutcome, ProgressPhaseCmd, StartTimerPhaseCmd, progress_phase,
    start_timer_phase,
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

fn make_test_config() -> Config {
    Config {
        timer: TimerConfiguration::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
        .expect("Failed to create timer configuration"),
        ..Config::default()
    }
}

/// Drive a task from start through one work session and its following break.
///
/// Uses `progress_phase` (the same usecase invoked by `CountdownExpiredHandler`
/// in production) so that cycling logic runs inline. Returns the `PhaseOutcome`
/// from the break-phase progression so callers can assert on it.
async fn complete_one_full_session(
    ctx: &crate::AppContext,
    task_id: domain::TaskId,
) -> PhaseOutcome {
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

    // Work phase expired → progress to break
    let from_phase = get_timer(ctx).await.get_current_phase();
    progress_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        ProgressPhaseCmd {
            task_id,
            from_phase,
        },
    )
    .await
    .expect("Failed to progress from work phase");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Break phase expired → progress to work (cycling happens here)
    let from_phase = get_timer(ctx).await.get_current_phase();
    let outcome = progress_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        ProgressPhaseCmd {
            task_id,
            from_phase,
        },
    )
    .await
    .expect("Failed to progress from break phase");

    // Let async event handlers settle
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    outcome
}

#[tokio::test]
async fn auto_advance_cycles_when_task_completes_with_auto_start() {
    let ctx = setup_minimal_ctx(
        "auto_advance_cycles_when_task_completes_with_auto_start",
    )
    .await;

    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, true),
    )
    .await;

    let test_config = make_test_config();

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

    let outcome = complete_one_full_session(&ctx, task1.id()).await;

    // Assert: outcome shows cycling to task2 with auto-start
    match outcome {
        PhaseOutcome::Started {
            cycled_to: Some(to_id),
            ..
        } => assert_eq!(to_id, task2.id()),
        other => panic!("Expected Started with cycled_to, got {other:?}"),
    }

    // Active task switched to task2
    let active = get_active_task(&ctx).await;
    assert_eq!(active.id(), task2.id());

    // task1 should be completed
    let task1_final =
        ctx.task_repo.get_by_id(task1.id()).await.unwrap().unwrap();
    assert!(task1_final.is_completed());

    // Timer should be running on task2
    let timer = get_timer(&ctx).await;
    assert!(timer.is_running());
    assert_eq!(timer.task_id(), Some(task2.id()));
}

#[tokio::test]
async fn auto_advance_cycles_even_when_auto_start_disabled() {
    let ctx =
        setup_minimal_ctx("auto_advance_cycles_even_when_auto_start_disabled")
            .await;

    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, false),
    )
    .await;

    let test_config = make_test_config();

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

    let outcome = complete_one_full_session(&ctx, task1.id()).await;

    // Assert: cycling still happened (Paused outcome, cycled to task2)
    match outcome {
        PhaseOutcome::Paused {
            cycled_to: Some(to_id),
            ..
        } => assert_eq!(to_id, task2.id()),
        other => panic!("Expected Paused with cycled_to, got {other:?}"),
    }

    let active = get_active_task(&ctx).await;
    assert_eq!(
        active.id(),
        task2.id(),
        "AutoAdvance must cycle to task2 even when auto_start_work_after_break is disabled"
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

    let test_config = make_test_config();

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

    let outcome = complete_one_full_session(&ctx, task1.id()).await;

    match outcome {
        PhaseOutcome::Started {
            cycled_to: Some(to_id),
            ..
        } => assert_eq!(
            to_id,
            task3.id(),
            "Should skip completed task2 and land on task3"
        ),
        other => panic!("Expected Started with cycled_to, got {other:?}"),
    }

    let active = get_active_task(&ctx).await;
    assert_eq!(active.id(), task3.id());
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

    let test_config = make_test_config();

    let task1 = TaskBuilder::new()
        .name("Only Task")
        .max_sessions(1)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task1.clone()).await.unwrap();

    let outcome = complete_one_full_session(&ctx, task1.id()).await;

    // Should be Stopped — no more tasks to cycle to
    assert!(matches!(outcome, PhaseOutcome::Stopped { .. }));

    let timer = get_timer(&ctx).await;
    assert_eq!(
        timer.task_id(),
        Some(task1.id()),
        "Timer should remain on task1 when there are no other tasks to cycle to"
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

    let test_config = make_test_config();

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

    let outcome = complete_one_full_session(&ctx, task1.id()).await;

    // In Manual mode, no cycling — outcome should not have cycled_to
    match &outcome {
        PhaseOutcome::Started {
            cycled_to: None, ..
        }
        | PhaseOutcome::Paused {
            cycled_to: None, ..
        } => {}
        other => panic!("Expected no cycling in Manual mode, got {other:?}"),
    }

    let active = get_active_task(&ctx).await;
    assert_eq!(
        active.id(),
        task1.id(),
        "Manual mode must not auto-advance even when another task is available"
    );
}

#[tokio::test]
async fn cycled_to_payload_identifies_from_and_to_tasks() {
    let ctx =
        setup_minimal_ctx("cycled_to_payload_identifies_from_and_to_tasks")
            .await;

    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, true),
    )
    .await;

    let test_config = make_test_config();

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

    let outcome = complete_one_full_session(&ctx, task1.id()).await;

    // The outcome's cycled_to identifies the next task.
    // The from_task_id is the original task_id passed to progress_phase.
    match outcome {
        PhaseOutcome::Started {
            cycled_to: Some(to_id),
            ..
        } => {
            assert_eq!(
                to_id,
                task2.id(),
                "cycled_to should match the next task"
            );
        }
        other => panic!("Expected Started with cycled_to, got {other:?}"),
    }
}
