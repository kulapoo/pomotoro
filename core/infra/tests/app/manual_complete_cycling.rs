//! Manual task-completion auto-advance.
//!
//! These tests mirror the orchestration performed by the Tauri
//! `complete_task` command (which lives in `apps/tauri-app` and therefore
//! cannot be unit-tested here): when a user force-completes a task and
//! `task_cycling_behavior == AutoAdvance`, the active task should switch to
//! the next incomplete task (round-robin). The new task's work phase
//! auto-starts when `auto_start_work_after_break` is true.
//!
//! The call sequence below is the *exact* sequence the command performs, so a
//! regression here is a regression in the user-facing "Complete" button.

use std::sync::Arc;
use std::time::Duration;

use domain::{
    Config, ConfigRepository, EventPublisher, TaskCyclingBehavior, TaskId,
    TaskRepository, TaskStatus, TimerConfiguration, event_names,
    task::CycleService,
};
use usecases::task::{SwitchActiveTaskCmd, complete_task, switch_active_task};
use usecases::timer::{
    StartTimerPhaseCmd, reset_timer_to_idle, start_timer_phase,
};

use crate::{
    AppContextBuilder, TaskBuilder,
    utils::{assert_utils, task::get_active_task, timer::get_timer},
};

/// Minimal context (no standard fixtures) so the round-robin selector only
/// sees the tasks each test creates explicitly.
async fn setup_minimal_ctx(name: &str) -> crate::AppContext {
    AppContextBuilder::new()
        .with_name(name)
        .build()
        .await
        .expect("Failed to build test context")
}

/// Build a cycling config with explicit auto-start flags.
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

/// Replicates the Tauri `complete_task` command's auto-advance branch.
///
/// 1. Force-complete the task via the use case.
/// 2. Reset the timer to idle for the completed task.
/// 3. If AutoAdvance is configured, pick the next task via `CycleService`,
///    switch to it, reset its timer, and — when `auto_start_work_after_break`
///    is set — start its work phase.
///
/// Returns the task id the timer is bound to after the sequence.
async fn manual_complete_with_auto_advance(
    ctx: &crate::AppContext,
    task_id: TaskId,
) -> TaskId {
    // 1. Force-complete.
    let task_repo_dyn: Arc<dyn TaskRepository + Send + Sync> =
        ctx.task_repo.clone();
    let event_bus_dyn: Arc<dyn EventPublisher + Send + Sync> =
        ctx.event_bus.clone();
    complete_task(&task_repo_dyn, &event_bus_dyn, task_id)
        .await
        .expect("Failed to complete task");

    // 2. Reset the timer to idle for the completed task (the command does
    //    this unconditionally before deciding to cycle).
    reset_timer_to_idle(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("Failed to reset timer to idle");

    let config = ctx.config_repo.get_config().await.unwrap();

    if !CycleService::should_auto_cycle(&config.general) {
        return task_id;
    }

    let active_tasks = ctx.task_repo.get_active_tasks().await.unwrap();
    let Some(next) = CycleService::select_next_task(
        &active_tasks,
        None,
        &config.general.task_cycling_behavior,
    ) else {
        return task_id;
    };
    let next_id = next.id();

    switch_active_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        SwitchActiveTaskCmd {
            task_id: next_id,
            old_task_id: Some(task_id),
        },
    )
    .await
    .expect("Failed to switch active task");

    reset_timer_to_idle(
        next_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("Failed to reset timer to idle for next task");

    if config.general.auto_start_work_after_break {
        start_timer_phase(
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_bus.clone(),
            StartTimerPhaseCmd {
                task_id: Some(next_id),
            },
        )
        .await
        .expect("Failed to auto-start next task work phase");
    }

    next_id
}

#[tokio::test]
async fn manual_complete_auto_advances_and_starts_when_config_enabled() {
    let ctx =
        setup_minimal_ctx("manual_complete_auto_advances_and_starts").await;
    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, true),
    )
    .await;

    let test_config = make_test_config();
    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(4)
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

    let final_id = manual_complete_with_auto_advance(&ctx, task1.id()).await;

    // Active task switched to task2.
    assert_eq!(final_id, task2.id());
    let active = get_active_task(&ctx).await;
    assert_eq!(active.id(), task2.id());

    // task1 completed.
    let task1_final =
        ctx.task_repo.get_by_id(task1.id()).await.unwrap().unwrap();
    assert!(task1_final.is_completed());

    // Auto-start honored: timer is running on task2.
    let timer = get_timer(&ctx).await;
    assert_eq!(timer.task_id(), Some(task2.id()));
    assert!(timer.is_running());
}

#[tokio::test]
async fn manual_complete_auto_advances_but_stays_idle_when_auto_start_disabled()
{
    let ctx = setup_minimal_ctx(
        "manual_complete_auto_advances_but_stays_idle_when_auto_start_disabled",
    )
    .await;
    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, false),
    )
    .await;

    let test_config = make_test_config();
    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(4)
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

    let final_id = manual_complete_with_auto_advance(&ctx, task1.id()).await;

    // Cycling still happened...
    assert_eq!(final_id, task2.id());
    let active = get_active_task(&ctx).await;
    assert_eq!(active.id(), task2.id());

    // ...but the new task's work phase must NOT auto-start.
    let timer = get_timer(&ctx).await;
    assert_eq!(timer.task_id(), Some(task2.id()));
    assert!(
        !timer.is_running(),
        "auto_start_work_after_break=false must leave the cycled task idle"
    );
}

#[tokio::test]
async fn manual_complete_skips_completed_tasks_round_robin() {
    let ctx = setup_minimal_ctx("manual_complete_skips_completed").await;
    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, false),
    )
    .await;

    let test_config = make_test_config();
    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task1.clone()).await.unwrap();

    // task2 is already completed — must be skipped by the round-robin selector.
    let task2 = TaskBuilder::new()
        .name("Task 2")
        .as_completed()
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

    let final_id = manual_complete_with_auto_advance(&ctx, task1.id()).await;

    // Should skip task2 and land on task3.
    assert_eq!(final_id, task3.id());
    let active = get_active_task(&ctx).await;
    assert_eq!(active.id(), task3.id());
}

#[tokio::test]
async fn manual_complete_noop_when_no_incomplete_tasks() {
    let ctx =
        setup_minimal_ctx("manual_complete_noop_when_no_incomplete_tasks")
            .await;
    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, true),
    )
    .await;

    let test_config = make_test_config();
    let task1 = TaskBuilder::new()
        .name("Only Task")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .config(test_config.clone())
        .build();
    ctx.task_repo.create(task1.clone()).await.unwrap();

    let final_id = manual_complete_with_auto_advance(&ctx, task1.id()).await;

    // No other task to cycle to — stays on task1.
    assert_eq!(final_id, task1.id());

    // The command unconditionally resets the timer to idle on complete, which
    // clears the task binding (idle = no task). Cycling must not have run.
    let timer = get_timer(&ctx).await;
    assert!(
        timer.task_id().is_none(),
        "Timer should be fully idle (no task bound) when there is nothing to cycle to"
    );
    assert!(!timer.is_running());

    let task1_final =
        ctx.task_repo.get_by_id(task1.id()).await.unwrap().unwrap();
    assert!(task1_final.is_completed());
}

#[tokio::test]
async fn manual_complete_does_not_auto_advance_in_manual_mode() {
    let ctx = setup_minimal_ctx(
        "manual_complete_does_not_auto_advance_in_manual_mode",
    )
    .await;
    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::Manual, true, true),
    )
    .await;

    let test_config = make_test_config();
    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(4)
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

    let final_id = manual_complete_with_auto_advance(&ctx, task1.id()).await;

    // Manual mode: no cycling — stays on the completed task1.
    assert_eq!(
        final_id,
        task1.id(),
        "Manual mode must not auto-advance on manual complete"
    );

    let task1_final =
        ctx.task_repo.get_by_id(task1.id()).await.unwrap().unwrap();
    assert!(task1_final.is_completed());

    // task2 untouched.
    let task2_final =
        ctx.task_repo.get_by_id(task2.id()).await.unwrap().unwrap();
    assert_eq!(task2_final.status(), TaskStatus::Active);
    assert!(!task2_final.is_completed());
}

#[tokio::test]
async fn manual_complete_emits_task_completed_event() {
    let ctx =
        setup_minimal_ctx("manual_complete_emits_task_completed_event").await;
    apply_config(
        &ctx,
        &cycling_config(TaskCyclingBehavior::AutoAdvance, true, false),
    )
    .await;

    let test_config = make_test_config();
    let task1 = TaskBuilder::new()
        .name("Task 1")
        .max_sessions(4)
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

    let _ = manual_complete_with_auto_advance(&ctx, task1.id()).await;

    // Let async event handlers settle.
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    // The force-complete must surface the task:task_completed UI event so the
    // frontend Tasks page refreshes.
    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::task::TASK_COMPLETED,
    );
}
