use std::sync::Arc;
use std::time::Duration;

use domain::{
    Config, EventPublisher, TaskRepository, TaskStatus, TimerRepository,
    event_names,
};

use crate::{AppContextBuilder, TaskBuilder};

/// Batch `reset_tasks` must publish a single `task:tasks_reset` UI event
/// (one toast) and MUST NOT emit any per-task `task:task_reset` events
/// (which would spam N toasts). Verified end-to-end through the
/// `TasksResetHandler` registered in `register_task_handlers`.
#[tokio::test]
async fn reset_tasks_batch_emits_single_tasks_reset_event() {
    let ctx = AppContextBuilder::new()
        .with_name("reset_tasks_batch_emits_single_tasks_reset_event")
        .build()
        .await
        .expect("build ctx");

    let task1 = TaskBuilder::new()
        .name("Batch A")
        .max_sessions(3)
        .status(TaskStatus::Completed)
        .current_sessions(3)
        .config(Config::default())
        .build();
    let task2 = TaskBuilder::new()
        .name("Batch B")
        .max_sessions(2)
        .status(TaskStatus::Completed)
        .current_sessions(2)
        .config(Config::default())
        .build();
    let id1 = task1.id();
    let id2 = task2.id();
    ctx.task_repo.create(task1).await.unwrap();
    ctx.task_repo.create(task2).await.unwrap();

    use usecases::task::reset_tasks;
    let task_repo_dyn: Arc<dyn TaskRepository + Send + Sync> =
        ctx.task_repo.clone();
    let timer_repo_dyn: Arc<dyn TimerRepository + Send + Sync> =
        ctx.timer_repo.clone();
    let event_bus_dyn: Arc<dyn EventPublisher + Send + Sync> =
        ctx.event_bus.clone();
    reset_tasks(task_repo_dyn, timer_repo_dyn, event_bus_dyn, vec![id1, id2])
        .await
        .expect("reset_tasks failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let batch = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::TASKS_RESET);
    assert_eq!(
        batch.len(),
        1,
        "expected exactly one task:tasks_reset event for a batch reset"
    );

    let per_task = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::TASK_RESET);
    assert!(
        per_task.is_empty(),
        "batch reset must not emit per-task task:task_reset events (would spam toasts)"
    );
}

/// Completing the last incomplete task must emit `task:tasks_completed`.
#[tokio::test]
async fn completing_last_task_emits_tasks_completed() {
    let ctx = AppContextBuilder::new()
        .with_name("completing_last_task_emits_tasks_completed")
        .build()
        .await
        .expect("build ctx");

    let task = TaskBuilder::new()
        .name("Only one")
        .max_sessions(2)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    let task_id = task.id();
    ctx.task_repo.create(task).await.unwrap();

    use usecases::task::complete_task;
    let task_repo_dyn: Arc<dyn TaskRepository + Send + Sync> =
        ctx.task_repo.clone();
    let event_bus_dyn: Arc<dyn EventPublisher + Send + Sync> =
        ctx.event_bus.clone();
    complete_task(&task_repo_dyn, &event_bus_dyn, task_id)
        .await
        .expect("complete_task failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::TASKS_COMPLETED);
    assert!(
        !events.is_empty(),
        "completing the last task should emit task:tasks_completed"
    );
}

/// Completing a task while others remain incomplete MUST NOT emit
/// `task:tasks_completed`.
#[tokio::test]
async fn completing_when_tasks_remain_does_not_emit_tasks_completed() {
    let ctx = AppContextBuilder::new()
        .with_name("completing_when_tasks_remain_does_not_emit_tasks_completed")
        .build()
        .await
        .expect("build ctx");

    let task1 = TaskBuilder::new()
        .name("First")
        .max_sessions(2)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    let task1_id = task1.id();
    ctx.task_repo.create(task1).await.unwrap();

    let task2 = TaskBuilder::new()
        .name("Second")
        .max_sessions(2)
        .status(TaskStatus::Queued)
        .config(Config::default())
        .build();
    ctx.task_repo.create(task2).await.unwrap();

    use usecases::task::complete_task;
    let task_repo_dyn: Arc<dyn TaskRepository + Send + Sync> =
        ctx.task_repo.clone();
    let event_bus_dyn: Arc<dyn EventPublisher + Send + Sync> =
        ctx.event_bus.clone();
    complete_task(&task_repo_dyn, &event_bus_dyn, task1_id)
        .await
        .expect("complete_task failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::TASKS_COMPLETED);
    assert!(
        events.is_empty(),
        "task:tasks_completed must not fire when incomplete tasks remain"
    );
}
