use std::time::Duration;

use domain::{Config, TaskRepository, TaskStatus, event_names};
use usecases::timer::{
    StartTimerPhaseCmd, pause_timer_phase, reset_timer_phase,
    resume_timer_phase, start_timer_phase,
};

use crate::{TaskBuilder, utils::setup::setup_ctx};

/// Helper: start a fresh timer bound to a task. Returns the task id.
async fn start_timer_for_task(ctx: &crate::AppContext) -> domain::TaskId {
    let task = TaskBuilder::new()
        .name("Payload test")
        .max_sessions(4)
        .status(TaskStatus::Active)
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
    .expect("start_timer_phase failed");

    tokio::time::sleep(Duration::from_millis(150)).await;
    task_id
}

#[tokio::test]
async fn timer_started_payload_carries_task_id_and_state() {
    let ctx =
        setup_ctx("timer_started_payload_carries_task_id_and_state").await;
    let task_id = start_timer_for_task(&ctx).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::START);
    assert!(!events.is_empty(), "timer:timer_started was not emitted");

    let payload = &events[0].payload;
    let embedded_task_id =
        payload.get("task_id").expect("payload missing task_id");
    assert_eq!(
        *embedded_task_id,
        serde_json::json!(task_id.to_string()),
        "payload task_id must match the started task"
    );
    assert!(
        payload.get("state").is_some(),
        "payload missing state field"
    );
}

#[tokio::test]
async fn timer_paused_payload_carries_task_id_and_state() {
    let ctx = setup_ctx("timer_paused_payload_carries_task_id_and_state").await;
    let task_id = start_timer_for_task(&ctx).await;

    pause_timer_phase(
        task_id,
        750,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("pause_timer_phase failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::PAUSE);
    assert!(!events.is_empty(), "timer:timer_paused was not emitted");

    let payload = &events[0].payload;
    assert_eq!(
        *payload.get("task_id").expect("payload missing task_id"),
        serde_json::json!(task_id.to_string())
    );
    assert!(payload.get("state").is_some());
}

#[tokio::test]
async fn timer_resumed_payload_carries_task_id_and_state() {
    let ctx =
        setup_ctx("timer_resumed_payload_carries_task_id_and_state").await;
    let task_id = start_timer_for_task(&ctx).await;

    // Pause, then resume — resume must emit timer:timer_resumed (not
    // timer:timer_started) with { task_id, state }.
    pause_timer_phase(
        task_id,
        750,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("pause_timer_phase failed");
    tokio::time::sleep(Duration::from_millis(150)).await;

    ctx.ui_simulator.app_handle().clear_events();

    resume_timer_phase(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("resume_timer_phase failed");
    tokio::time::sleep(Duration::from_millis(150)).await;

    let resume_events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::RESUME);
    assert!(
        !resume_events.is_empty(),
        "timer:timer_resumed was not emitted"
    );

    let payload = &resume_events[0].payload;
    assert_eq!(
        *payload.get("task_id").expect("payload missing task_id"),
        serde_json::json!(task_id.to_string())
    );
    assert!(payload.get("state").is_some());

    // Negative assertion: resume must NOT have emitted timer:timer_started
    // (it now uses a distinct Resumed domain event).
    let started_events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::START);
    assert!(
        started_events.is_empty(),
        "resume must not emit timer:timer_started; expected timer:timer_resumed only"
    );
}

#[tokio::test]
async fn timer_reset_payload_carries_task_id_and_state() {
    let ctx = setup_ctx("timer_reset_payload_carries_task_id_and_state").await;
    let task_id = start_timer_for_task(&ctx).await;

    reset_timer_phase(
        task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("reset_timer_phase failed");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = ctx
        .ui_simulator
        .app_handle()
        .events_of_type(event_names::ui_listeners::timer::RESET);
    assert!(!events.is_empty(), "timer:timer_reset was not emitted");

    let payload = &events[0].payload;
    assert_eq!(
        *payload.get("task_id").expect("payload missing task_id"),
        serde_json::json!(task_id.to_string())
    );
    assert!(payload.get("state").is_some());
}
