use std::time::Duration;
use std::sync::Arc;
use pomotoro_lib::timer::TimerService;
use pomotoro_lib::task::{InMemoryTaskRepository, TaskRepository};
use pomotoro_lib::config::{InMemoryConfigRepo, ConfigRepository};
use pomotoro_lib::timer::models::{Phase, TimerStatus};
use pomotoro_lib::task::models::TaskStatus;

// Helper function to create test context
async fn create_test_context() -> (Arc<TimerService>, TaskRepository, ConfigRepository) {
    let timer_manager = Arc::new(TimerService::new());
    let task_repo: TaskRepository = Arc::new(InMemoryTaskRepository::with_default_task());
    let config_repo: ConfigRepository = Arc::new(InMemoryConfigRepo::new());

    (timer_manager, task_repo, config_repo)
}

#[tokio::test]
async fn test_timer_initial_state() {
    let (timer_manager, _task_repo, _config_repo) = create_test_context().await;

    let state = timer_manager.get_state().await;

    assert_eq!(state.phase, Phase::Work);
    assert_eq!(state.status, TimerStatus::Stopped);
    assert_eq!(state.session_count, 0);
    assert_eq!(state.task_session_count, 0);
    assert_eq!(state.remaining_seconds, 25 * 60);
}

#[tokio::test]
async fn test_start_timer() {
    let (timer_manager, task_repo, _config_repo) = create_test_context().await;

    // Get default task
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    // Switch to the task and set status to running
    timer_manager.switch_task(default_task.id, Some(default_task)).await;
    timer_manager.set_status(TimerStatus::Running).await;

    let state = timer_manager.get_state().await;

    assert_eq!(state.status, TimerStatus::Running);
    assert_eq!(state.phase, Phase::Work);
    assert_eq!(state.remaining_seconds, 25 * 60);
}

#[tokio::test]
async fn test_pause_timer() {
    let (timer_manager, task_repo, _config_repo) = create_test_context().await;

    // Get default task and start timer
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_manager.switch_task(default_task.id, Some(default_task)).await;
    timer_manager.set_status(TimerStatus::Running).await;

    // Wait a bit then pause
    tokio::time::sleep(Duration::from_millis(100)).await;
    timer_manager.set_status(TimerStatus::Paused).await;

    let state = timer_manager.get_state().await;

    assert_eq!(state.status, TimerStatus::Paused);
    assert_eq!(state.phase, Phase::Work);
    // Time should be slightly less than 25 minutes (allowing for some variance)
    assert!(state.remaining_seconds <= 25 * 60);
}

#[tokio::test]
async fn test_reset_timer() {
    let (timer_manager, task_repo, _config_repo) = create_test_context().await;

    // Get default task and start timer
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_manager.switch_task(default_task.id, Some(default_task)).await;
    timer_manager.set_status(TimerStatus::Running).await;

    // Wait a bit then reset
    tokio::time::sleep(Duration::from_millis(100)).await;
    timer_manager.reset_current_phase(Some(default_task)).await;
    timer_manager.set_status(TimerStatus::Stopped).await;

    let state = timer_manager.get_state().await;

    assert_eq!(state.status, TimerStatus::Stopped);
    assert_eq!(state.phase, Phase::Work);
    assert_eq!(state.remaining_seconds, 25 * 60);
}

#[tokio::test]
async fn test_task_repository_basic_operations() {
    let (_timer_manager, task_repo, _config_repo) = create_test_context().await;

    // Test get all tasks
    let tasks = task_repo.get_all().await.unwrap();
    assert_eq!(tasks.len(), 1); // Should have default task

    let default_task = &tasks[0];
    assert_eq!(default_task.name, "Focus Session");
    assert_eq!(default_task.status, TaskStatus::Active);
    assert_eq!(default_task.max_sessions, 4);
    assert_eq!(default_task.current_sessions, 0);
}

#[tokio::test]
async fn test_config_repository_basic_operations() {
    let (_timer_manager, _task_repo, config_repo) = create_test_context().await;

    // Test get config
    let config = config_repo.get_config().unwrap();

    // Test default configuration values
    assert_eq!(config.default_task_config.work_duration, Duration::from_secs(25 * 60));
    assert_eq!(config.default_task_config.short_break_duration, Duration::from_secs(5 * 60));
    assert_eq!(config.default_task_config.long_break_duration, Duration::from_secs(15 * 60));
    assert_eq!(config.default_task_config.sessions_until_long_break, 4);
}

#[tokio::test]
async fn test_timer_state_consistency() {
    let (timer_manager, task_repo, _config_repo) = create_test_context().await;

    // Get initial state
    let initial_state = timer_manager.get_state().await;
    assert_eq!(initial_state.status, TimerStatus::Stopped);

    // Start timer
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    timer_manager.switch_task(default_task.id, Some(default_task)).await;
    timer_manager.set_status(TimerStatus::Running).await;

    let running_state = timer_manager.get_state().await;
    assert_eq!(running_state.status, TimerStatus::Running);
    assert_eq!(running_state.active_task_id, Some(default_task.id));

    // Get state again to verify consistency
    let current_state = timer_manager.get_state().await;
    assert_eq!(current_state.status, TimerStatus::Running);
    assert_eq!(current_state.active_task_id, Some(default_task.id));
}

#[tokio::test]
async fn test_basic_workflow() {
    let (timer_manager, task_repo, _config_repo) = create_test_context().await;

    // 1. Get default task
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    // 2. Start timer
    timer_manager.switch_task(default_task.id, Some(default_task)).await;
    timer_manager.set_status(TimerStatus::Running).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.status, TimerStatus::Running);

    // 3. Pause timer
    timer_manager.set_status(TimerStatus::Paused).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.status, TimerStatus::Paused);

    // 4. Resume timer
    timer_manager.set_status(TimerStatus::Running).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.status, TimerStatus::Running);

    // 5. Reset timer
    timer_manager.reset_current_phase(Some(default_task)).await;
    timer_manager.set_status(TimerStatus::Stopped).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.status, TimerStatus::Stopped);
    assert_eq!(state.remaining_seconds, 25 * 60);
}