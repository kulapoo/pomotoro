use crate::task::models::TaskTestRepository;
use crate::timer::models::{TimerTestService, TimerTestAssertions};
use domain::{Phase, TaskRepository};
use std::time::Duration;

#[tokio::test]
async fn test_e2e_complete_work_session() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await?;
    let default_task = &tasks[0];

    // Start work session
    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await?;
    
    let running_state = timer_service.get_state().await;
    TimerTestAssertions::assert_is_running(&running_state);
    TimerTestAssertions::assert_is_work_phase(&running_state);
    
    // Skip to break (simulating completed work session)
    let (old_phase, new_phase) = timer_service.skip_to_next_phase(Some(default_task)).await?;
    assert_eq!(old_phase, Phase::Work);
    assert_eq!(new_phase, Phase::ShortBreak);
    
    let break_state = timer_service.get_state().await;
    assert_eq!(break_state.phase(), Phase::ShortBreak);
    assert_eq!(break_state.session_count(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_e2e_full_pomodoro_cycle() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await?;
    let default_task = &tasks[0];
    timer_service.setup_with_task(default_task).await;

    // Complete 4 work sessions to trigger long break
    for session in 1..=4 {
        // Start work session
        timer_service.start_work_session().await?;
        let work_state = timer_service.get_state().await;
        TimerTestAssertions::assert_is_work_phase(&work_state);
        
        // Complete work session
        let (old_phase, new_phase) = timer_service.skip_to_next_phase(Some(default_task)).await?;
        assert_eq!(old_phase, Phase::Work);
        
        if session < 4 {
            assert_eq!(new_phase, Phase::ShortBreak);
            
            // Complete short break
            let (break_old, break_new) = timer_service.skip_to_next_phase(Some(default_task)).await?;
            assert_eq!(break_old, Phase::ShortBreak);
            assert_eq!(break_new, Phase::Work);
        } else {
            // 4th session should trigger long break
            assert_eq!(new_phase, Phase::LongBreak);
        }
        
        let state = timer_service.get_state().await;
        assert_eq!(state.session_count(), session);
    }
    
    // Verify we're in long break after 4 sessions
    let final_state = timer_service.get_state().await;
    assert_eq!(final_state.phase(), Phase::LongBreak);
    assert_eq!(final_state.session_count(), 4);
    
    Ok(())
}

#[tokio::test]
async fn test_e2e_timer_persistence() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await?;
    let default_task = &tasks[0];

    // Set up initial state
    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await?;
    
    // Simulate some time passing
    timer_service.wait_for_seconds(1).await;
    
    // Pause and get current state
    timer_service.pause_timer().await?;
    let paused_state = timer_service.get_state().await;
    
    // Resume and verify state is preserved
    timer_service.start_work_session().await?;
    let resumed_state = timer_service.get_state().await;
    
    assert_eq!(resumed_state.phase(), paused_state.phase());
    assert_eq!(resumed_state.session_count(), paused_state.session_count());
    assert_eq!(resumed_state.active_task_id, paused_state.active_task_id);
    // Remaining time should be close (within tolerance for test timing)
    let time_diff = (resumed_state.remaining_seconds() as i64 - paused_state.remaining_seconds() as i64).abs();
    assert!(time_diff <= 1, "Time difference too large: {time_diff}");
    
    Ok(())
}

#[tokio::test]
async fn test_e2e_timer_with_multiple_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::new();
    
    // Create and seed multiple tasks
    let task_ids = task_repo.seed_with_test_data().await?;
    let work_task_id = task_ids[0];
    let study_task_id = task_ids[1];
    
    let work_task = task_repo.get_by_id(work_task_id).await?.unwrap();
    let study_task = task_repo.get_by_id(study_task_id).await?.unwrap();
    
    // Work on first task
    timer_service.setup_with_task(&work_task).await;
    timer_service.start_work_session().await?;
    
    let work_state = timer_service.get_state().await;
    TimerTestAssertions::assert_has_active_task(&work_state, work_task_id);
    TimerTestAssertions::assert_is_running(&work_state);
    
    // Complete a session
    timer_service.skip_to_next_phase(Some(&work_task)).await?;
    let after_work = timer_service.get_state().await;
    assert_eq!(after_work.session_count(), 1);
    
    // Switch to study task
    timer_service.setup_with_task(&study_task).await;
    let study_state = timer_service.get_state().await;
    TimerTestAssertions::assert_has_active_task(&study_state, study_task_id);
    
    // Session count should be preserved globally
    assert_eq!(study_state.session_count(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_e2e_timer_reset_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await?;
    let default_task = &tasks[0];

    // Start timer and then reset
    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await?;
    
    let initial_state = timer_service.get_state().await;
    assert_eq!(initial_state.phase(), Phase::Work);
    
    // Reset current phase
    timer_service.reset_current_phase(Some(default_task)).await?;
    
    let reset_state = timer_service.get_state().await;
    assert_eq!(reset_state.phase(), Phase::Work);
    assert_eq!(reset_state.remaining_seconds(), 25 * 60);
    assert_eq!(reset_state.session_count(), initial_state.session_count()); // Session count preserved
    
    Ok(())
}

#[tokio::test]
async fn test_e2e_timer_performance_stress() -> Result<(), Box<dyn std::error::Error>> {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await?;
    let default_task = &tasks[0];
    timer_service.setup_with_task(default_task).await;

    let start_time = std::time::Instant::now();
    
    // Rapidly start/stop timer multiple times
    for _ in 0..100 {
        timer_service.start_work_session().await?;
        timer_service.pause_timer().await?;
        timer_service.start_work_session().await?;
        timer_service.stop_timer().await?;
    }
    
    let elapsed = start_time.elapsed();
    assert!(elapsed < Duration::from_millis(1000), "Timer operations took too long: {elapsed:?}");
    
    // Verify final state is consistent
    let final_state = timer_service.get_state().await;
    TimerTestAssertions::assert_is_stopped(&final_state);
    
    Ok(())
}