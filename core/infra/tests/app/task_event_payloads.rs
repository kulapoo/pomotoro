use std::sync::Arc;
use std::time::Duration;

use domain::{
    Config, ConfigRepository, EventPublisher, Phase, TaskRepository,
    TaskStatus, event_names,
};
use usecases::task::complete_task;

use crate::{AppContextBuilder, TaskBuilder, utils::setup::setup_ctx};

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

/// `task:auto_advanced` payload must carry the full next Task so the React
/// EventBus can `set({ activeTask: payload.to_task })` directly. Verified
/// via the CountdownExpiredHandler path (the cycle source for auto-advance).
#[tokio::test]
async fn auto_advanced_payload_embeds_to_task() {
    use domain::{TaskCyclingBehavior, timer::events::CountdownExpired};
    use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

    let ctx = AppContextBuilder::new()
        .with_name("auto_advanced_payload_embeds_to_task")
        .build()
        .await
        .expect("build ctx");

    // AutoAdvance + auto_start_work_after_break so the cycle fires. Field
    // names verified at core/domain/src/config/general.rs:11-14 — all on
    // `config.general` (no `cycling` sub-struct).
    let mut cfg = Config::default();
    cfg.general.task_cycling_behavior = TaskCyclingBehavior::AutoAdvance;
    cfg.general.auto_start_breaks = true;
    cfg.general.auto_start_work_after_break = true;
    ctx.config_repo.save_config(&cfg).await.unwrap();

    let task1 = TaskBuilder::new()
        .name("First")
        .max_sessions(1)
        .status(TaskStatus::Active)
        .config(cfg.clone())
        .build();
    let task1_id = task1.id();
    ctx.task_repo.create(task1).await.unwrap();

    let task2 = TaskBuilder::new()
        .name("Second")
        .max_sessions(2)
        .status(TaskStatus::Active)
        .config(cfg.clone())
        .build();
    let task2_id = task2.id();
    let task2_name = task2.name().to_string();
    ctx.task_repo.create(task2).await.unwrap();

    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerPhaseCmd {
            task_id: Some(task1_id),
        },
    )
    .await
    .expect("start");

    tokio::time::sleep(Duration::from_millis(100)).await;
    ctx.ui_simulator.app_handle().clear_events();

    // First expiry: Work phase ends. task1's only session is consumed
    // (max_sessions=1 → status becomes Completed), timer auto-transitions
    // to ShortBreak (auto_start_breaks = true). Cycling does NOT happen
    // here — `progress_phase` only cycles when from_phase is a break.
    ctx.event_bus
        .publish(Box::new(CountdownExpired::new(Phase::Work, task1_id)));
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Second expiry: ShortBreak ends. `finish_break` finalizes task1 and
    // AutoAdvance cycles to task2 → emits task:auto_advanced.
    ctx.event_bus
        .publish(Box::new(CountdownExpired::new(Phase::ShortBreak, task1_id)));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::task::AUTO_ADVANCED);
    assert!(!events.is_empty(), "task:auto_advanced was not emitted");

    let payload = &events[0].payload;
    let embedded = payload
        .get("to_task")
        .expect("payload missing `to_task` field");
    assert_eq!(embedded["id"], task2_id.to_string());
    assert_eq!(embedded["name"], task2_name);
}

/// `timer:phase_completed` payload must carry both the new timer state
/// AND the updated bound Task so the React EventBus can update both
/// slices without an IPC round-trip. Verified via the natural-expiry
/// path (CountdownExpiredHandler Started arm).
#[tokio::test]
async fn phase_completed_payload_embeds_task_and_timer() {
    use domain::{Phase, timer::events::CountdownExpired};
    use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

    let ctx = AppContextBuilder::new()
        .with_name("phase_completed_payload_embeds_task_and_timer")
        .build()
        .await
        .expect("build ctx");

    let task = TaskBuilder::new()
        .name("Embed me")
        .max_sessions(3)
        .status(TaskStatus::Active)
        .config(Config::default())
        .build();
    let task_id = task.id();
    let expected_name = task.name().to_string();
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
    .expect("start");

    tokio::time::sleep(Duration::from_millis(100)).await;
    ctx.ui_simulator.app_handle().clear_events();

    // Natural work-phase expiry drives CountdownExpiredHandler Started arm.
    ctx.event_bus
        .publish(Box::new(CountdownExpired::new(Phase::Work, task_id)));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::timer::PHASE_COMPLETED);
    assert!(!events.is_empty(), "timer:phase_completed was not emitted");

    let payload = &events[0].payload;
    let timer = payload
        .get("timer")
        .expect("payload missing `timer` envelope field");
    assert!(
        timer.get("state").is_some(),
        "timer field must carry TimerState"
    );
    let task = payload
        .get("task")
        .expect("payload missing `task` envelope field");
    assert_eq!(task["id"], task_id.to_string());
    assert_eq!(task["name"], expected_name);
}
