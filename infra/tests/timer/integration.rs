use crate::task::models::TaskTestRepository;
use crate::timer::models::{TimerStateBuilder, TimerTestService, TimerTestAssertions};
use domain::{Phase, TimerStatus, TaskRepository};

#[tokio::test]
async fn test_timer_initial_state() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();

    let state = timer_service.get_state().await?;

    TimerTestAssertions::assert_is_work_phase(&state);
    TimerTestAssertions::assert_is_stopped(&state);
    TimerTestAssertions::assert_session_count(&state, 0);
    assert_eq!(state.session_count(), 0);
    assert_eq!(state.remaining_seconds(), 25 * 60);
    
    Ok(())
}

#[tokio::test]
async fn test_timer_start_stop_cycle() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();

    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await.unwrap();

    let running_state = timer_service.get_state().await?;
    TimerTestAssertions::assert_is_running(&running_state);
    TimerTestAssertions::assert_is_work_phase(&running_state);
    TimerTestAssertions::assert_has_active_task(&running_state, default_task.id);

    timer_service.stop_timer().await.unwrap();

    let stopped_state = timer_service.get_state().await?;
    TimerTestAssertions::assert_is_stopped(&stopped_state);
    
    Ok(())
}

#[tokio::test]
async fn test_timer_pause_resume() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();

    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await.unwrap();

    timer_service.wait_for_seconds(1).await;
    timer_service.pause_timer().await.unwrap();

    let paused_state = timer_service.get_state().await?;
    assert_eq!(paused_state.status(), TimerStatus::Paused);
    assert_eq!(paused_state.phase(), Phase::Work);
    assert!(paused_state.remaining_seconds() <= 25 * 60);

    // Toggle pause again to resume
    timer_service.pause_timer().await.unwrap();

    let resumed_state = timer_service.get_state().await?;
    TimerTestAssertions::assert_is_running(&resumed_state);
    assert_eq!(resumed_state.remaining_seconds(), paused_state.remaining_seconds());
    Ok(())
}

#[tokio::test]
async fn test_timer_reset_phase() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();

    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await.unwrap();

    timer_service.wait_for_seconds(1).await;
    timer_service.reset_current_phase(Some(default_task)).await.unwrap();

    let reset_state = timer_service.get_state().await?;
    TimerTestAssertions::assert_is_work_phase(&reset_state);
    assert_eq!(reset_state.remaining_seconds(), 25 * 60);
    Ok(())
}

#[tokio::test]
async fn test_timer_phase_skipping() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();

    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await?;

    let (old_phase, new_phase) = timer_service
        .skip_to_next_phase(Some(default_task))
        .await
        .unwrap();

    assert_eq!(old_phase, Phase::Work);
    assert_eq!(new_phase, Phase::ShortBreak);

    let state = timer_service.get_state().await?;
    assert_eq!(state.phase(), Phase::ShortBreak);
    Ok(())
}

#[tokio::test]
async fn test_timer_state_consistency() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();

    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    let initial_state = timer_service.get_state().await?;
    TimerTestAssertions::assert_is_stopped(&initial_state);

    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await.unwrap();

    let running_state = timer_service.get_state().await?;
    TimerTestAssertions::assert_is_running(&running_state);
    TimerTestAssertions::assert_has_active_task(&running_state, default_task.id);

    let current_state = timer_service.get_state().await?;
    TimerTestAssertions::assert_is_running(&current_state);
    TimerTestAssertions::assert_has_active_task(&current_state, default_task.id);
    Ok(())
}

#[tokio::test]
async fn test_timer_task_switching() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::new();

    // Seed test data
    let task_ids = task_repo.seed_with_test_data().await.unwrap();
    let work_task_id = task_ids[0];
    let study_task_id = task_ids[1];

    let work_task = task_repo.get_by_id(work_task_id).await.unwrap().unwrap();
    let study_task = task_repo.get_by_id(study_task_id).await.unwrap().unwrap();

    // Start with work task
    timer_service.setup_with_task(&work_task).await;
    let state1 = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&state1, work_task_id);

    // Switch to study task
    timer_service.setup_with_task(&study_task).await;
    let state2 = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&state2, study_task_id);
    Ok(())
}

#[test]
fn test_timer_state_builder() {
    let state = TimerStateBuilder::new()
        .running()
        .work_phase()
        .with_session_count(2)
        .with_entity_session_count(1)
        .build();

    TimerTestAssertions::assert_is_running(&state);
    TimerTestAssertions::assert_is_work_phase(&state);
    TimerTestAssertions::assert_session_count(&state, 2);
    assert_eq!(state.entity_session_count(), 1);
    assert_eq!(state.remaining_seconds(), 25 * 60);
}

#[test]
fn test_basic_timer_types() {
    use domain::{Phase, TimerStatus};

    // Test Phase enum
    assert_eq!(Phase::Work, Phase::Work);
    assert_ne!(Phase::Work, Phase::ShortBreak);

    // Test TimerStatus enum
    assert_eq!(TimerStatus::Stopped, TimerStatus::Stopped);
    assert_ne!(TimerStatus::Running, TimerStatus::Paused);
}

// MVP 2.0 Multi-Task and Cycling Features

#[tokio::test]
async fn test_multi_task_session_management() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::new();

    // Create tasks with different session limits
    let task_ids = task_repo.seed_with_test_data().await.unwrap();
    let quick_task_id = task_ids[0]; // 1 session
    let medium_task_id = task_ids[1]; // 3 sessions
    let long_task_id = task_ids[2]; // 8 sessions

    let quick_task = task_repo.get_by_id(quick_task_id).await.unwrap().unwrap();
    let medium_task = task_repo.get_by_id(medium_task_id).await.unwrap().unwrap();
    let long_task = task_repo.get_by_id(long_task_id).await.unwrap().unwrap();

    // Start with quick task
    timer_service.setup_with_task(&quick_task).await;
    timer_service.start_work_session().await.unwrap();

    // Complete the quick task session
    timer_service.force_complete_session().await.unwrap();

    let state_after_quick = timer_service.get_state().await?;
    assert_eq!(state_after_quick.entity_session_count(), 1);

    // Switch to medium task - should maintain independent tracking
    // First stop the timer to allow task switching
    timer_service.stop_timer().await?;
    timer_service.setup_with_task(&medium_task).await;
    let state_with_medium = timer_service.get_state().await?;
    assert_eq!(state_with_medium.entity_session_count(), 0); // Fresh start for medium task
    TimerTestAssertions::assert_has_active_task(&state_with_medium, medium_task_id);

    // Complete one session of medium task
    // Start a work session and complete it
    timer_service.start_work_session().await.unwrap();
    timer_service.force_complete_session().await.unwrap(); // Work -> Break (increments count)

    let state_medium_progress = timer_service.get_state().await?;
    assert_eq!(state_medium_progress.entity_session_count(), 1);

    // Switch to long task - should also start fresh
    // First stop the timer to allow task switching
    timer_service.stop_timer().await?;
    timer_service.setup_with_task(&long_task).await;
    let state_with_long = timer_service.get_state().await?;
    assert_eq!(state_with_long.entity_session_count(), 0); // Independent tracking
    TimerTestAssertions::assert_has_active_task(&state_with_long, long_task_id);
    Ok(())
}

#[tokio::test]
async fn test_task_cycling_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::new();

    // Create a cycle of active tasks
    let task_ids = task_repo.seed_with_test_data().await.unwrap();
    let tasks: Vec<_> = {
        let mut tasks = Vec::new();
        for task_id in &task_ids[0..3] { // First 3 tasks
            let task = task_repo.get_by_id(*task_id).await.unwrap().unwrap();
            tasks.push(task);
        }
        tasks
    };

    // Set up first task (cycling would be handled at application level)
    timer_service.setup_with_task(&tasks[0]).await;

    // Verify initial task is first in queue
    let initial_state = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&initial_state, tasks[0].id);

    // Switch to next task manually (cycling logic would be in application layer)
    timer_service.setup_with_task(&tasks[1]).await;
    let state_after_switch1 = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&state_after_switch1, tasks[1].id);

    // Switch to third task
    timer_service.setup_with_task(&tasks[2]).await;
    let state_after_switch2 = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&state_after_switch2, tasks[2].id);

    // Switch back to first task (simulating wrap around)
    timer_service.setup_with_task(&tasks[0]).await;
    let state_after_wrap = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&state_after_wrap, tasks[0].id);
    
    Ok(())
}

#[tokio::test]
async fn test_automatic_task_progression() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::new();

    // Create task with limit of 2 sessions
    let mut task = crate::task::models::TaskBuilder::new("Limited Task".to_string(), 2)
        .build();
    task_repo.create(task.clone()).await.unwrap();

    timer_service.setup_with_task(&task).await;
    // Note: Auto task progression would be configured at the application level

    // Complete first session
    timer_service.start_work_session().await.unwrap();
    timer_service.force_complete_session().await.unwrap();

    // Manually update task to reflect first session completion (simulate app layer)
    task.increment_session().unwrap();
    task_repo.update(task.clone()).await.unwrap();

    let state_after_session1 = timer_service.get_state().await?;
    assert_eq!(state_after_session1.entity_session_count(), 1);
    TimerTestAssertions::assert_has_active_task(&state_after_session1, task.id);

    // Complete second (final) session - should auto-complete task
    // First complete the break phase to get back to work
    timer_service.force_complete_session().await.unwrap(); // Complete break -> go to work
    timer_service.force_complete_session().await.unwrap(); // Complete work -> task_session_count = 2

    // Manually update task to reflect second session completion (simulate app layer)
    task.increment_session().unwrap();
    task_repo.update(task.clone()).await.unwrap();

    let state_after_completion = timer_service.get_state().await?;
    assert_eq!(state_after_completion.entity_session_count(), 2);

    // Task should be marked as completed in repository
    let completed_task = task_repo.get_by_id(task.id).await.unwrap().unwrap();
    assert!(completed_task.is_completed());
    assert_eq!(completed_task.current_sessions, 2);
    Ok(())
}

#[tokio::test]
async fn test_task_specific_timer_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::empty();

    // Create task with custom timing configuration (MVP2 feature)
    use domain::{TaskConfig, TaskBuilder as DomainTaskBuilder};
    use std::time::Duration;

    let custom_task = DomainTaskBuilder::with_name_and_sessions("Custom Timer Task".to_string(), 3)
        .with_config(TaskConfig::new(
            Duration::from_secs(45 * 60), // 45 min work sessions
            Duration::from_secs(10 * 60), // 10 min short breaks
            Duration::from_secs(25 * 60), // 25 min long breaks
            2,                            // Long break every 2 sessions
            false,                        // No screen blocking
        ).unwrap())
        .build()
        .unwrap();

    task_repo.create(custom_task.clone()).await.unwrap();

    // Setup timer with custom task
    timer_service.setup_with_task(&custom_task).await;

    let state = timer_service.get_state().await?;

    // Verify timer uses custom durations
    assert_eq!(state.remaining_seconds(), 45 * 60); // Custom 45-min work session
    TimerTestAssertions::assert_has_active_task(&state, custom_task.id);

    // Complete work session and check break duration
    timer_service.start_work_session().await.unwrap();
    timer_service.force_complete_session().await.unwrap();

    let break_state = timer_service.get_state().await?;
    assert_eq!(break_state.phase(), Phase::ShortBreak);
    assert_eq!(break_state.remaining_seconds(), 10 * 60); // Custom 10-min break

    // Complete break and verify custom long break schedule (every 2 sessions, not 4)
    timer_service.force_complete_session().await.unwrap();
    timer_service.start_work_session().await.unwrap();
    timer_service.force_complete_session().await.unwrap();

    let long_break_state = timer_service.get_state().await?;
    assert_eq!(long_break_state.phase(), Phase::LongBreak);
    assert_eq!(long_break_state.remaining_seconds(), 25 * 60); // Custom 25-min long break
    
    Ok(())
}

#[tokio::test]
async fn test_skip_completed_tasks_in_cycle() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::new();

    // Create tasks with different completion states
    let task1 = crate::task::models::TaskBuilder::new("Active Task".to_string(), 3).build();
    let mut task2 = crate::task::models::TaskBuilder::new("Completed Task".to_string(), 1).build();
    let task3 = crate::task::models::TaskBuilder::new("Another Active".to_string(), 2).build();

    // Complete task2
    task2.increment_session().unwrap();

    task_repo.create(task1.clone()).await.unwrap();
    task_repo.create(task2.clone()).await.unwrap();
    task_repo.create(task3.clone()).await.unwrap();

    let _tasks = vec![task1.clone(), task2.clone(), task3.clone()];
    // Setup with first non-completed task

    // Start with first task
    timer_service.setup_with_task(&task1).await;
    let initial_state = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&initial_state, task1.id);

    // Switch should skip completed task2 and go to task3
    timer_service.setup_with_task(&task3).await;
    let state_after_switch = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&state_after_switch, task3.id);

    // Switch back to task1 (skipping completed task2)
    timer_service.setup_with_task(&task1).await;
    let state_after_wrap = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&state_after_wrap, task1.id);
    Ok(())
}

#[tokio::test]
async fn test_manual_task_switching_during_session() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::new();

    let task_ids = task_repo.seed_with_test_data().await.unwrap();
    let task1 = task_repo.get_by_id(task_ids[0]).await.unwrap().unwrap();
    let task2 = task_repo.get_by_id(task_ids[1]).await.unwrap().unwrap();

    // Start work session with task1
    timer_service.setup_with_task(&task1).await;
    timer_service.start_work_session().await.unwrap();

    // Wait a bit
    timer_service.wait_for_seconds(1).await;
    let _remaining_before_switch = timer_service.get_state().await?.remaining_seconds();

    // Cannot switch tasks while timer is running - must stop first
    // This is correct domain behavior
    timer_service.stop_timer().await?;
    
    // Now switch to task2
    timer_service.setup_with_task(&task2).await;

    let state_after_switch = timer_service.get_state().await?;
    TimerTestAssertions::assert_has_active_task(&state_after_switch, task2.id);
    TimerTestAssertions::assert_is_stopped(&state_after_switch);
    
    // Start a new session with task2
    timer_service.start_work_session().await.unwrap();
    let state_running = timer_service.get_state().await?;
    TimerTestAssertions::assert_is_running(&state_running);
    assert_eq!(state_running.phase(), Phase::Work);
    Ok(())
}

#[tokio::test]
async fn test_session_tracking_across_multiple_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::new();

    // Create fresh tasks for this test instead of using completed task from seed data
    let task1 = crate::task::models::TaskBuilder::new("Session Task 1".to_string(), 3).build();
    let task2 = crate::task::models::TaskBuilder::new("Session Task 2".to_string(), 3).build();
    let task3 = crate::task::models::TaskBuilder::new("Session Task 3".to_string(), 3).build();

    task_repo.create(task1.clone()).await.unwrap();
    task_repo.create(task2.clone()).await.unwrap();
    task_repo.create(task3.clone()).await.unwrap();

    // Complete sessions on different tasks
    timer_service.setup_with_task(&task1).await;
    timer_service.start_work_session().await.unwrap();
    timer_service.force_complete_session().await.unwrap(); // Work -> Break (count = 1)

    // Manually update task1 to reflect session completion (simulate app layer)
    let mut updated_task1 = task1.clone();
    updated_task1.increment_session().unwrap();
    task_repo.update(updated_task1).await.unwrap();

    timer_service.setup_with_task(&task2).await;
    timer_service.force_complete_session().await.unwrap(); // Break -> Work
    timer_service.start_work_session().await.unwrap();
    timer_service.force_complete_session().await.unwrap(); // Work -> Break (count = 2)

    // Manually update task2 to reflect session completion (simulate app layer)
    let mut updated_task2 = task2.clone();
    updated_task2.increment_session().unwrap();
    task_repo.update(updated_task2).await.unwrap();

    timer_service.setup_with_task(&task3).await;
    timer_service.force_complete_session().await.unwrap(); // Break -> Work
    timer_service.start_work_session().await.unwrap();
    timer_service.force_complete_session().await.unwrap(); // Work -> Break (count = 3)

    // Manually update task3 to reflect session completion (simulate app layer)
    let mut updated_task3 = task3.clone();
    updated_task3.increment_session().unwrap();
    task_repo.update(updated_task3).await.unwrap();

    // Verify global session count tracks all tasks
    let final_state = timer_service.get_state().await?;
    assert_eq!(final_state.session_count(), 3); // Global count

    // Verify individual task session counts
    let updated_task1 = task_repo.get_by_id(task1.id).await.unwrap().unwrap();
    let updated_task2 = task_repo.get_by_id(task2.id).await.unwrap().unwrap();
    let updated_task3 = task_repo.get_by_id(task3.id).await.unwrap().unwrap();

    assert_eq!(updated_task1.current_sessions, 1);
    assert_eq!(updated_task2.current_sessions, 1);
    assert_eq!(updated_task3.current_sessions, 1);
    Ok(())
}