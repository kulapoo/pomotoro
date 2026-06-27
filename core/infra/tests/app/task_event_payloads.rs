use std::sync::Arc;
use std::time::Duration;

use domain::{Config, EventPublisher, TaskRepository, TaskStatus, event_names};
use usecases::task::complete_task;

use crate::{TaskBuilder, utils::setup::setup_ctx};

/// `task:task_completed` payload must carry the full Task object (not just
/// audit fields) so the React EventBus can direct-map it into the
/// `activeTask` store slice without an IPC round-trip.
#[tokio::test]
async fn task_completed_payload_embeds_full_task() {
    let ctx = setup_ctx("task_completed_payload_embeds_full_task").await;

    let task = TaskBuilder::new()
        .name("Embed me")
        .max_sessions(3)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    let task_id = task.id();
    let expected_name = task.name().to_string();
    let expected_max = task.max_sessions();
    ctx.task_repo.create(task).await.unwrap();

    // `complete_task` signature: `(task_repo: &Arc<dyn ...>, event_publisher:
    // &Arc<dyn ...>, task_id: TaskId)`. See core/usecases/src/task/complete_task.rs.
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
        .events_of_type(event_names::task::TASK_COMPLETED);
    assert!(!events.is_empty(), "task:task_completed was not emitted");

    let payload = &events[0].payload;
    let embedded = payload.get("task").expect("payload missing `task` field");
    assert_eq!(embedded["name"], expected_name);
    assert_eq!(embedded["max_sessions"], expected_max);
    assert_eq!(embedded["status"], "Completed");
    assert_eq!(
        embedded["id"],
        task_id.to_string(),
        "embedded task id must match the completed task"
    );
}

/// `task:active_changed` payload must carry the full new active Task so the
/// React EventBus can `set({ activeTask: payload.task })` directly.
#[tokio::test]
async fn active_changed_payload_embeds_full_task() {
    let ctx = setup_ctx("active_changed_payload_embeds_full_task").await;

    let task1 = TaskBuilder::new()
        .name("First")
        .max_sessions(2)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    ctx.task_repo.create(task1).await.unwrap();

    let task2 = TaskBuilder::new()
        .name("Second")
        .max_sessions(4)
        .status(TaskStatus::Queued)
        .config(Config::default())
        .build();
    let task2_id = task2.id();
    let task2_name = task2.name().to_string();
    ctx.task_repo.create(task2).await.unwrap();

    // Mark task1 as the current active by binding it to the timer (the
    // switch_active_task usecase reads the prior active from the timer).
    // Then switch to task2.
    use usecases::task::{SwitchActiveTaskCmd, switch_active_task};
    switch_active_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        SwitchActiveTaskCmd {
            task_id: task2_id,
            old_task_id: None,
        },
    )
    .await
    .expect("switch_active_task failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::ACTIVE_CHANGED);
    assert!(!events.is_empty(), "task:active_changed was not emitted");

    let payload = &events[0].payload;
    let embedded = payload.get("task").expect("payload missing `task` field");
    assert_eq!(embedded["id"], task2_id.to_string());
    assert_eq!(embedded["name"], task2_name);
    assert_eq!(embedded["max_sessions"], 4);
}

/// `task:task_reset` payload must carry the full reset Task so the React
/// EventBus can reconcile `activeTask` directly (when the reset task is the
/// active one).
#[tokio::test]
async fn task_reset_payload_embeds_full_task() {
    let ctx = setup_ctx("task_reset_payload_embeds_full_task").await;

    let task = TaskBuilder::new()
        .name("Reset me")
        .max_sessions(3)
        .status(TaskStatus::Active)
        .current_sessions(2)
        .config(Config::default())
        .build();
    let task_id = task.id();
    let task_name = task.name().to_string();
    ctx.task_repo.create(task).await.unwrap();

    use usecases::task::reset_task;
    reset_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        task_id,
    )
    .await
    .expect("reset_task failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::TASK_RESET);
    assert!(!events.is_empty(), "task:task_reset was not emitted");

    let payload = &events[0].payload;
    let embedded = payload.get("task").expect("payload missing `task` field");
    assert_eq!(embedded["id"], task_id.to_string());
    assert_eq!(embedded["name"], task_name);
    assert_eq!(embedded["current_sessions"], 0);
}
