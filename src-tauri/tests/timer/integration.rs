use crate::task::models::TaskTestRepository;
use crate::timer::models::{TimerStateBuilder, TimerTestService, TimerTestAssertions};
use pomotoro_lib::timer::models::{Phase, TimerStatus};
use pomotoro_lib::task::TaskRepositoryTrait;

#[tokio::test]
async fn test_timer_initial_state() {
    let timer_service = TimerTestService::new();
    
    let state = timer_service.get_state().await;

    TimerTestAssertions::assert_is_work_phase(&state);
    TimerTestAssertions::assert_is_stopped(&state);
    TimerTestAssertions::assert_session_count(&state, 0);
    assert_eq!(state.task_session_count, 0);
    assert_eq!(state.remaining_seconds, 25 * 60);
}

#[tokio::test]
async fn test_timer_start_stop_cycle() {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await.unwrap();

    let running_state = timer_service.get_state().await;
    TimerTestAssertions::assert_is_running(&running_state);
    TimerTestAssertions::assert_is_work_phase(&running_state);
    TimerTestAssertions::assert_has_active_task(&running_state, default_task.id);

    timer_service.stop_timer().await.unwrap();

    let stopped_state = timer_service.get_state().await;
    TimerTestAssertions::assert_is_stopped(&stopped_state);
}

#[tokio::test]
async fn test_timer_pause_resume() {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await.unwrap();

    timer_service.wait_for_seconds(1).await;
    timer_service.pause_timer().await.unwrap();

    let paused_state = timer_service.get_state().await;
    assert_eq!(paused_state.status, TimerStatus::Paused);
    assert_eq!(paused_state.phase, Phase::Work);
    assert!(paused_state.remaining_seconds <= 25 * 60);

    timer_service.start_work_session().await.unwrap();

    let resumed_state = timer_service.get_state().await;
    TimerTestAssertions::assert_is_running(&resumed_state);
    assert_eq!(resumed_state.remaining_seconds, paused_state.remaining_seconds);
}

#[tokio::test]
async fn test_timer_reset_phase() {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await.unwrap();

    timer_service.wait_for_seconds(1).await;
    timer_service.reset_current_phase(Some(default_task)).await.unwrap();

    let reset_state = timer_service.get_state().await;
    TimerTestAssertions::assert_is_work_phase(&reset_state);
    assert_eq!(reset_state.remaining_seconds, 25 * 60);
}

#[tokio::test]
async fn test_timer_phase_skipping() {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_service.setup_with_task(default_task).await;

    let (old_phase, new_phase) = timer_service
        .skip_to_next_phase(Some(default_task))
        .await
        .unwrap();

    assert_eq!(old_phase, Phase::Work);
    assert_eq!(new_phase, Phase::ShortBreak);

    let state = timer_service.get_state().await;
    assert_eq!(state.phase, Phase::ShortBreak);
}

#[tokio::test]
async fn test_timer_state_consistency() {
    let timer_service = TimerTestService::new();
    let task_repo = TaskTestRepository::with_default_task();
    
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    let initial_state = timer_service.get_state().await;
    TimerTestAssertions::assert_is_stopped(&initial_state);

    timer_service.setup_with_task(default_task).await;
    timer_service.start_work_session().await.unwrap();

    let running_state = timer_service.get_state().await;
    TimerTestAssertions::assert_is_running(&running_state);
    TimerTestAssertions::assert_has_active_task(&running_state, default_task.id);

    let current_state = timer_service.get_state().await;
    TimerTestAssertions::assert_is_running(&current_state);
    TimerTestAssertions::assert_has_active_task(&current_state, default_task.id);
}

#[tokio::test]
async fn test_timer_task_switching() {
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
    let state1 = timer_service.get_state().await;
    TimerTestAssertions::assert_has_active_task(&state1, work_task_id);

    // Switch to study task
    timer_service.setup_with_task(&study_task).await;
    let state2 = timer_service.get_state().await;
    TimerTestAssertions::assert_has_active_task(&state2, study_task_id);
}

#[test]
fn test_timer_state_builder() {
    let state = TimerStateBuilder::new()
        .running()
        .work_phase()
        .with_session_count(2)
        .with_task_session_count(1)
        .build();

    TimerTestAssertions::assert_is_running(&state);
    TimerTestAssertions::assert_is_work_phase(&state);
    TimerTestAssertions::assert_session_count(&state, 2);
    assert_eq!(state.task_session_count, 1);
    assert_eq!(state.remaining_seconds, 25 * 60);
}

#[test]
fn test_basic_timer_types() {
    use pomotoro_lib::timer::models::{Phase, TimerStatus};

    // Test Phase enum
    assert_eq!(Phase::Work, Phase::Work);
    assert_ne!(Phase::Work, Phase::ShortBreak);

    // Test TimerStatus enum
    assert_eq!(TimerStatus::Stopped, TimerStatus::Stopped);
    assert_ne!(TimerStatus::Running, TimerStatus::Paused);
}