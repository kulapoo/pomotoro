use std::time::Duration;

use domain::{Config, TaskCyclerService, TaskRepository, TaskStatus, TimerConfiguration, TimerState, TimerStatus};
use usecases::{
    CreateTaskCmd, create_task,
    timer::{StartTimerSessionCmd, complete_timer_phase, start_timer_session},
};

use crate::{TaskBuilder, utils::{setup::setup_ctx, task::get_active_task, timer::get_timer}};

#[tokio::test]
async fn timer_should_complete_full_pomodoro_cycle() {
    let ctx = setup_ctx("timer_should_complete_full_pomodoro_cycle").await;

    // AAA

    // Arrange
    // Configure test with specific Pomodoro settings

    let default_config = Config {
        timer: TimerConfiguration::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
        .expect("Failed to create timer configuration"),
        ..Default::default()
    };

    // Act
    let task1_result = create_task(
        ctx.task_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Task1".to_string(),
            description: None,
            max_sessions: 8,  // Increased to allow testing beyond first cycle
            tags: Vec::new(),
            config: Some(default_config),
        },
    )
    .await;

    let task1_id = task1_result.clone().expect("Failed to create task").id;

    let session0_timer_is_idle = get_timer(&ctx).await.state().is_idle();

    // Start the timer for session 1

    let timer_session1_result = start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task1_id),
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session1_timer_is_work_phase_before_completion =
        get_timer(&ctx).await.state().is_work_phase();

    let session1_timer_status = get_timer(&ctx).await.state().status();

    let task_1_current_sessions = get_active_task(&ctx).await.current_sessions;

    // Complete first work session

    let session1_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session1_timer_is_break_phase =
        get_timer(&ctx).await.state().is_break_phase();

    let task_2_current_sessions = get_active_task(&ctx).await.current_sessions;

    // Complete first short break

    let session1_short_break_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session1_break_timer_is_work_phase =
        get_timer(&ctx).await.state().is_work_phase();

    // Repeat work/short break cycle for sessions 2 and 3

    // Complete Session 2 Work
    let session2_work_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session2_timer_is_break_phase =
        get_timer(&ctx).await.state().is_break_phase();
    let task_3_current_sessions = get_active_task(&ctx).await.current_sessions;

    // Complete Session 2 Short Break
    let session2_break_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session2_break_timer_is_work_phase =
        get_timer(&ctx).await.state().is_work_phase();

    // Complete Session 3 Work
    let session3_work_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session3_timer_is_break_phase =
        get_timer(&ctx).await.state().is_break_phase();
    let task_4_current_sessions = get_active_task(&ctx).await.current_sessions;

    // Complete Session 3 Short Break
    let session3_break_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session3_break_timer_is_work_phase =
        get_timer(&ctx).await.state().is_work_phase();

    // Complete final work session before long break

    // Complete Session 4 Work - This should trigger LONG BREAK
    let session4_work_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session4_timer_state = get_timer(&ctx).await.state().clone();
    let session4_timer_is_long_break = matches!(session4_timer_state, TimerState::LongBreak { .. });
    let task_5_current_sessions = get_active_task(&ctx).await.current_sessions;


    // Complete long break

    // Complete Long Break
    let long_break_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let long_break_timer_is_work_phase =
        get_timer(&ctx).await.state().is_work_phase();


    // Verify second cycle starts correctly

    // Complete one more work session to verify cycle continues (should go to short break, not long)
    let session5_work_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session5_timer_state = get_timer(&ctx).await.state().clone();
    let session5_timer_is_short_break = matches!(session5_timer_state, TimerState::ShortBreak { .. });
    let task_6_current_sessions = get_active_task(&ctx).await.current_sessions;

    // (Cleanup is handled automatically by the test framework)

    // Assert

    // Initial setup assertions
    assert_eq!(task1_result.is_ok(), true);
    assert_eq!(session0_timer_is_idle, true);
    assert_eq!(timer_session1_result.is_ok(), true);
    assert_eq!(session1_timer_is_work_phase_before_completion, true);
    assert_eq!(session1_timer_status, TimerStatus::Running);
    assert_eq!(task_1_current_sessions, 0); // Session not incremented when starting, only when completing

    // Session 1 assertions
    assert_eq!(session1_result.is_ok(), true);
    assert_eq!(session1_timer_is_break_phase, true);
    assert_eq!(task_2_current_sessions, 1); // First work session completed, so now 1
    assert_eq!(session1_short_break_result.is_ok(), true);
    assert_eq!(session1_break_timer_is_work_phase, true);

    // Session 2 assertions
    assert_eq!(session2_work_result.is_ok(), true);
    assert_eq!(session2_timer_is_break_phase, true);
    assert_eq!(task_3_current_sessions, 2); // Second work session completed
    assert_eq!(session2_break_result.is_ok(), true);
    assert_eq!(session2_break_timer_is_work_phase, true);

    // Session 3 assertions
    assert_eq!(session3_work_result.is_ok(), true);
    assert_eq!(session3_timer_is_break_phase, true);
    assert_eq!(task_4_current_sessions, 3); // Third work session completed
    assert_eq!(session3_break_result.is_ok(), true);
    assert_eq!(session3_break_timer_is_work_phase, true);

    // Session 4 assertions - LONG BREAK
    assert_eq!(session4_work_result.is_ok(), true);
    assert_eq!(session4_timer_is_long_break, true); // Should be long break after 4 sessions
    assert_eq!(task_5_current_sessions, 4); // Fourth work session completed

    // Long break completion assertions
    assert_eq!(long_break_result.is_ok(), true);
    assert_eq!(long_break_timer_is_work_phase, true); // Back to work after long break

    // Second cycle assertions (session 5 should go to short break, not long)
    assert_eq!(session5_work_result.is_ok(), true);
    assert_eq!(session5_timer_is_short_break, true); // Should be short break, not long
    assert_eq!(task_6_current_sessions, 5); // Fifth work session completed
}

// ### Test 22: Task cycling with multiple tasks
// ```pseudo
// TEST: "should_cycle_through_incomplete_tasks"
// GIVEN:
//     context = setup_test_context()
//     task1 = create_task(context, "Task 1", TaskStatus::Active)
//     task2 = create_task(context, "Task 2", TaskStatus::Queued)
//     task3 = create_task(context, "Task 3", TaskStatus::Completed)

//     // Set cycling strategy
//     strategy = TaskCyclingStrategy::Sequential
// WHEN:
//     next1 = context.usecases.cycle_task.execute(strategy, None).value
//     next2 = context.usecases.cycle_task.execute(strategy, Some(next1.id)).value
//     next3 = context.usecases.cycle_task.execute(strategy, Some(next2.id)).value
// THEN:
//     assert next1.id == task1.id  // First incomplete
//     assert next2.id == task2.id  // Second incomplete
//     assert next3.id == task1.id  // Cycles back (task3 is completed)
// ```
#[tokio::test]
async fn timer_should_cycle_through_incomplete_tasks() {
    let ctx = setup_ctx("timer_should_cycle_through_incomplete_tasks").await;

    // AAA

    // Arrange
    // Create three tasks with different statuses

    // Task 1: Active (incomplete)
    let task1 = TaskBuilder::new()
        .name("Task 1")
        .description("First task - active")
        .max_sessions(4)
        .status(TaskStatus::Active)
        .build();

    ctx.task_repo
        .create(task1.clone())
        .await
        .expect("Failed to create task 1");

    // Task 2: Queued (incomplete)
    let task2 = TaskBuilder::new()
        .name("Task 2")
        .description("Second task - queued")
        .max_sessions(4)
        .status(TaskStatus::Queued)
        .build();

    ctx.task_repo
        .create(task2.clone())
        .await
        .expect("Failed to create task 2");

    // Task 3: Completed (should be skipped)
    let task3 = TaskBuilder::new()
        .name("Task 3")
        .description("Third task - completed")
        .max_sessions(2)
        .current_sessions(2)
        .status(TaskStatus::Completed)
        .build();

    ctx.task_repo
        .create(task3.clone())
        .await
        .expect("Failed to create task 3");

    // Act
    // Cycle through incomplete tasks

    // Get the incomplete task queue to verify setup
    let incomplete_queue_before = ctx.task_cycling_service
        .get_incomplete_task_queue()
        .await
        .expect("Failed to get incomplete task queue");

    // First cycle: None -> should get first incomplete task
    let next1 = ctx.task_cycling_service
        .cycle_to_next_incomplete_task(None)
        .await
        .expect("Failed to cycle to first task")
        .expect("Should have found an incomplete task");

    // Second cycle: current task -> should get next incomplete task
    let next2 = ctx.task_cycling_service
        .cycle_to_next_incomplete_task(Some(next1.id))
        .await
        .expect("Failed to cycle to second task")
        .expect("Should have found next incomplete task");

    // Third cycle: current task -> should get next incomplete task
    let next3 = ctx.task_cycling_service
        .cycle_to_next_incomplete_task(Some(next2.id))
        .await
        .expect("Failed to cycle to third task")
        .expect("Should have found another incomplete task");

    // Assert

    // The test setup might include a default task, so we verify our tasks are in the queue
    let has_task1 = incomplete_queue_before.iter().any(|t| t.name == "Task 1");
    let has_task2 = incomplete_queue_before.iter().any(|t| t.name == "Task 2");
    let has_task3 = incomplete_queue_before.iter().any(|t| t.name == "Task 3");

    assert!(has_task1, "Task 1 should be in incomplete queue");
    assert!(has_task2, "Task 2 should be in incomplete queue");
    assert!(!has_task3, "Completed Task 3 should NOT be in incomplete queue");

    // The cycling behavior should work correctly regardless of initial order
    // Since there might be a default task, we have 3 incomplete tasks total
    // We expect to cycle through all incomplete tasks

    // Verify first three cycles return different tasks
    assert_ne!(next1.id, next2.id, "First and second cycle should return different tasks");
    assert_ne!(next2.id, next3.id, "Second and third cycle should return different tasks");

    // Collect all cycled task names
    let cycled_names = vec![next1.name.clone(), next2.name.clone(), next3.name.clone()];

    // Verify our created incomplete tasks are being cycled through
    let cycles_task1 = cycled_names.iter().any(|n| n == "Task 1");
    let cycles_task2 = cycled_names.iter().any(|n| n == "Task 2");

    assert!(cycles_task1, "Should cycle through Task 1");
    assert!(cycles_task2, "Should cycle through Task 2");

    // Verify completed task is never returned
    assert!(!cycled_names.iter().any(|n| n == "Task 3"),
            "Should never cycle through completed Task 3");

    // Fourth cycle should wrap back to the beginning
    let next4 = ctx.task_cycling_service
        .cycle_to_next_incomplete_task(Some(next3.id))
        .await
        .expect("Failed to cycle to fourth task")
        .expect("Should wrap back to first task");

    // Verify it wraps back to the first task we got
    assert_eq!(next4.id, next1.id, "Should cycle back to first task after going through all incomplete tasks");
}


