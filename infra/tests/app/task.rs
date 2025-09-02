use std::any::TypeId;
use std::sync::Arc;

use domain::{
    TaskCreated, TaskRepository, TaskStatus,
    event_names,
};
use usecases::{
    CreateTaskCmd, create_task,
};

use crate::AppContextBuilder;

#[tokio::test]

async fn should_create_task_with_title() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .build()
        .await
        .expect("Failed to build context");

    let event_bus = ctx.event_bus.clone();
    let simulator = (*ctx.ui_simulator).clone().start_listen_to_events();
    let task_repo = ctx.task_repo.clone();
    let task_created = create_task(
        ctx.task_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Test Task".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        },
    )
    .await
    .expect("Failed to create task");

    // Wait for async event handlers to complete
    // This is needed because event handlers run asynchronously
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let events = simulator.app_handle().emitted_events();

    let task = task_repo
        .get_by_id(task_created.id)
        .await
        .expect("Failed to get task")
        .expect("Task not found");

    assert_eq!(task.name, "Test Task");
    assert_eq!(task.status, TaskStatus::Queued);
    assert_eq!(task.current_sessions, 0);
    assert_eq!(task.max_sessions, 4);
    assert_eq!(task.tags, Vec::<String>::new());
    assert!(
        simulator
            .app_handle()
            .was_event_emitted(event_names::ui_listeners::task::LIST_UPDATED),
        "Expected task:list_updated event to be emitted, but got: {:?}",
        events.iter().map(|e| &e.event_name).collect::<Vec<_>>()
    );
    assert!(event_bus.has_event_type(TypeId::of::<TaskCreated>()));
}

// #[tokio::test]
// async fn tasks_should_have_unique_ids() {
//     let ctx = AppContextBuilder::new()
//         .with_standard_fixtures()
//         .build()
//         .await
//         .expect("Failed to build context");

//     // Create first task
//     let task1 = create_task(
//         ctx.task_repo.clone(),
//         ctx.timer_repo.clone(),
//         ctx.event_bus.clone(),
//         CreateTaskCmd {
//             name: "Task 1".to_string(),
//             description: None,
//             max_sessions: 4,
//             tags: vec![],
//         },
//     )
//     .await
//     .expect("Failed to create task 1");

//     // Create second task
//     let task2 = create_task(
//         ctx.task_repo.clone(),
//         ctx.timer_repo.clone(),
//         ctx.event_bus.clone(),
//         CreateTaskCmd {
//             name: "Task 2".to_string(),
//             description: None,
//             max_sessions: 4,
//             tags: vec![],
//         },
//     )
//     .await
//     .expect("Failed to create task 2");

//     // Verify IDs are different
//     assert_ne!(task1.id, task2.id, "Task IDs should be unique");

//     // Verify IDs are valid UUIDs (they should parse as strings without panic)
//     assert!(!task1.id.to_string().is_empty(), "Task 1 ID should be valid");
//     assert!(!task2.id.to_string().is_empty(), "Task 2 ID should be valid");
// }

// #[tokio::test]
// async fn should_find_task_by_id() {
//     let ctx = AppContextBuilder::new()
//         .with_standard_fixtures()
//         .build()
//         .await
//         .expect("Failed to build context");

//     // Create a task
//     let created_task = create_task(
//         ctx.task_repo.clone(),
//         ctx.timer_repo.clone(),
//         ctx.event_bus.clone(),
//         CreateTaskCmd {
//             name: "Find me".to_string(),
//             description: None,
//             max_sessions: 4,
//             tags: vec![],
//         },
//     )
//     .await
//     .expect("Failed to create task");

//     // Find the task by ID
//     let task_repo: Arc<dyn TaskRepository + Send + Sync> = ctx.task_repo.clone();
//     let found_task = get_task(
//         &task_repo,
//         GetTaskQuery {
//             id: created_task.id.to_string(),
//         },
//     )
//     .await
//     .expect("Failed to find task");

//     // Verify it's the same task
//     assert_eq!(found_task.id, created_task.id);
//     assert_eq!(found_task.name, "Find me");
// }

// #[tokio::test]
// async fn should_update_task_status() {
//     let ctx = AppContextBuilder::new()
//         .with_standard_fixtures()
//         .build()
//         .await
//         .expect("Failed to build context");

//     // Create a task
//     let task = create_task(
//         ctx.task_repo.clone(),
//         ctx.timer_repo.clone(),
//         ctx.event_bus.clone(),
//         CreateTaskCmd {
//             name: "Update my status".to_string(),
//             description: None,
//             max_sessions: 4,
//             tags: vec![],
//         },
//     )
//     .await
//     .expect("Failed to create task");

//     // Verify initial status is Queued
//     assert_eq!(task.status, TaskStatus::Queued);

//     // Update the task - change name which should trigger status change
//     let updated_task = update_task(
//         ctx.task_repo.clone(),
//         ctx.event_bus.clone(),
//         UpdateTaskCmd {
//             id: task.id.to_string(),
//             name: Some("Updated name".to_string()),
//             description: None,
//             max_sessions: None,
//             tags: None,
//             work_duration: None,
//             short_break_duration: None,
//             long_break_duration: None,
//             sessions_until_long_break: None,
//             enable_screen_blocking: None,
//             audio_config: None,
//         },
//     )
//     .await
//     .expect("Failed to update task");

//     // Verify the task was updated
//     assert_eq!(updated_task.name, "Updated name");

//     // Check if TaskUpdated event was published
//     tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
//     assert!(ctx.event_bus.has_event_type(TypeId::of::<TaskUpdated>()));
// }

// #[tokio::test]
// async fn should_delete_task() {
//     let ctx = AppContextBuilder::new()
//         .with_standard_fixtures()
//         .build()
//         .await
//         .expect("Failed to build context");

//     // Create a task
//     let task = create_task(
//         ctx.task_repo.clone(),
//         ctx.timer_repo.clone(),
//         ctx.event_bus.clone(),
//         CreateTaskCmd {
//             name: "Delete me".to_string(),
//             description: None,
//             max_sessions: 4,
//             tags: vec![],
//         },
//     )
//     .await
//     .expect("Failed to create task");

//     // Delete the task
//     let delete_result = delete_task(
//         ctx.task_repo.clone(),
//         ctx.event_bus.clone(),
//         DeleteTaskCmd {
//             id: task.id.to_string(),
//         },
//     )
//     .await
//     .expect("Failed to delete task");

//     assert!(delete_result, "Task should be deleted");

//     // Try to find the deleted task
//     let task_repo: Arc<dyn TaskRepository + Send + Sync> = ctx.task_repo.clone();
//     let find_result = get_task(
//         &task_repo,
//         GetTaskQuery {
//             id: task.id.to_string(),
//         },
//     )
//     .await;

//     // Should fail to find the task
//     assert!(find_result.is_err(), "Should not find deleted task");

//     // Verify the error message
//     if let Err(e) = find_result {
//         match e {
//             domain::Error::TaskNotFound { .. } => (),
//             _ => panic!("Expected TaskNotFound error, got: {:?}", e),
//         }
//     }
// }

// #[tokio::test]
// async fn should_track_task_timer_relationship() {
//     let ctx = AppContextBuilder::new()
//         .with_standard_fixtures()
//         .build()
//         .await
//         .expect("Failed to build context");

//     // Create a task
//     let task = create_task(
//         ctx.task_repo.clone(),
//         ctx.timer_repo.clone(),
//         ctx.event_bus.clone(),
//         CreateTaskCmd {
//             name: "Task with Timer".to_string(),
//             description: Some("Task to test timer relationship".to_string()),
//             max_sessions: 4,
//             tags: vec!["test".to_string()],
//         },
//     )
//     .await
//     .expect("Failed to create task");

//     // Verify task has a timer_id
//     let retrieved_task = ctx.task_repo
//         .get_by_id(task.id)
//         .await
//         .expect("Failed to get task")
//         .expect("Task not found");

//     // Timer ID should be set
//     assert!(!retrieved_task.get_timer_id().to_string().is_empty(), "Task should have a timer ID");

//     // Verify the timer exists for this task
//     let timer = ctx.timer_repo
//         .get_by_id(retrieved_task.get_timer_id())
//         .await
//         .expect("Failed to get timer")
//         .expect("Timer not found for task");

//     // Timer should be in idle state initially
//     assert!(timer.is_idle(), "Timer should be idle initially");
// }

// #[tokio::test]
// async fn task_sessions_should_update_with_timer() {
//     use usecases::timer::{StartTimerSessionCmd, start_timer_session};
//     use domain::timer::TimerService;

//     let ctx = AppContextBuilder::new()
//         .with_standard_fixtures()
//         .build()
//         .await
//         .expect("Failed to build context");

//     // Create a task with 2 max sessions for easier testing
//     let task = create_task(
//         ctx.task_repo.clone(),
//         ctx.timer_repo.clone(),
//         ctx.event_bus.clone(),
//         CreateTaskCmd {
//             name: "Session Tracking Task".to_string(),
//             description: None,
//             max_sessions: 2,
//             tags: vec![],
//         },
//     )
//     .await
//     .expect("Failed to create task");

//     // Initial task should have 0 sessions
//     let initial_task = ctx.task_repo
//         .get_by_id(task.id)
//         .await
//         .expect("Failed to get task")
//         .expect("Task not found");

//     assert_eq!(initial_task.current_sessions, 0, "Task should start with 0 sessions");
//     assert_eq!(initial_task.status, TaskStatus::Queued, "Task should be queued initially");

//     // Start timer for the task
//     start_timer_session(
//         ctx.task_repo.clone(),
//         ctx.timer_repo.clone(),
//         ctx.event_bus.clone(),
//         StartTimerSessionCmd {
//             task_id: Some(task.id.to_string()),
//         },
//     )
//     .await
//     .expect("Failed to start timer");

//     // Task status should now be Active
//     let active_task = ctx.task_repo
//         .get_by_id(task.id)
//         .await
//         .expect("Failed to get task")
//         .expect("Task not found");

//     assert_eq!(active_task.status, TaskStatus::Active, "Task should be active when timer starts");

//     // Verify timer is associated with the task
//     let timer_state = ctx.timer_service
//         .get_state()
//         .await
//         .expect("Failed to get timer state");

//     assert_eq!(
//         timer_state.active_entity_id(),
//         Some(task.id.to_string()),
//         "Timer should be associated with the task"
//     );
// }
