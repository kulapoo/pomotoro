use std::sync::Arc;
use std::time::Duration;

use pomotoro_lib::timer::TimerService;
use pomotoro_lib::task::{InMemoryTaskRepository, TaskRepository};
use pomotoro_lib::config::{InMemoryConfigRepo, ConfigRepository};
use pomotoro_lib::timer::models::{Phase, TimerStatus};
use pomotoro_lib::task::models::TaskStatus;

// Helper to create test context
fn create_test_context() -> (Arc<TimerService>, TaskRepository, ConfigRepository) {
    let timer_manager = Arc::new(TimerService::new());
    let task_repo: TaskRepository = Arc::new(InMemoryTaskRepository::with_default_task());
    let config_repo: ConfigRepository = Arc::new(InMemoryConfigRepo::new());

    (timer_manager, task_repo, config_repo)
}

#[tokio::test]
async fn test_timer_manager_creation() {
    let (timer_manager, _task_repo, _config_repo) = create_test_context();

    let state = timer_manager.get_state().await;

    // Verify initial state
    assert_eq!(state.status, TimerStatus::Stopped);
    assert_eq!(state.phase, Phase::Work);
    assert_eq!(state.session_count, 0);
    assert_eq!(state.remaining_seconds, 25 * 60); // 25 minutes
}

#[tokio::test]
async fn test_timer_status_changes() {
    let (timer_manager, _task_repo, _config_repo) = create_test_context();

    // Test setting status to Running
    timer_manager.set_status(TimerStatus::Running).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.status, TimerStatus::Running);

    // Test setting status to Paused
    timer_manager.set_status(TimerStatus::Paused).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.status, TimerStatus::Paused);

    // Test setting status to Stopped
    timer_manager.set_status(TimerStatus::Stopped).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.status, TimerStatus::Stopped);
}

#[tokio::test]
async fn test_timer_stop_functionality() {
    let (timer_manager, _task_repo, _config_repo) = create_test_context();

    // Set to running first
    timer_manager.set_status(TimerStatus::Running).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.status, TimerStatus::Running);

    // Stop timer
    timer_manager.stop_timer().await;

    // Need to explicitly set status to stopped
    timer_manager.set_status(TimerStatus::Stopped).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.status, TimerStatus::Stopped);
}

#[tokio::test]
async fn test_task_repository_default_task() {
    let (_timer_manager, task_repo, _config_repo) = create_test_context();

    let tasks = task_repo.get_all().await.unwrap();

    // Should have exactly one default task
    assert_eq!(tasks.len(), 1);

    let default_task = &tasks[0];
    assert_eq!(default_task.name, "Focus Session");
    assert_eq!(default_task.status, TaskStatus::Active);
    assert_eq!(default_task.max_sessions, 4);
    assert_eq!(default_task.current_sessions, 0);
    assert!(default_task.tags.contains(&"focus".to_string()));
}

#[tokio::test]
async fn test_config_repository_defaults() {
    let (_timer_manager, _task_repo, config_repo) = create_test_context();

    let config = config_repo.get_config().unwrap();

    // Test default timing values
    assert_eq!(config.default_task_config.work_duration, Duration::from_secs(25 * 60));
    assert_eq!(config.default_task_config.short_break_duration, Duration::from_secs(5 * 60));
    assert_eq!(config.default_task_config.long_break_duration, Duration::from_secs(15 * 60));
    assert_eq!(config.default_task_config.sessions_until_long_break, 4);
    assert_eq!(config.default_task_config.enable_screen_blocking, false);

    // Test default audio config
    assert_eq!(config.default_audio_config.volume, 0.7);
    assert_eq!(config.default_audio_config.enable_background_audio, false);
    assert_eq!(config.default_audio_config.muted, false);
}

#[tokio::test]
async fn test_task_switch_functionality() {
    let (timer_manager, task_repo, _config_repo) = create_test_context();

    // Get default task
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    // Test switching to a task
    timer_manager.switch_task(default_task.id, Some(default_task)).await;
    let state = timer_manager.get_state().await;
    assert_eq!(state.active_task_id, Some(default_task.id));
}

#[tokio::test]
async fn test_task_crud_operations() {
    let (_timer_manager, task_repo, _config_repo) = create_test_context();

    // Create a custom task
    let mut custom_task = pomotoro_lib::task::models::Task::new("Custom Task".to_string(), 2);
    custom_task = custom_task.with_tags(vec!["work".to_string(), "test".to_string()]);

    // Test create
    task_repo.create(custom_task.clone()).await.unwrap();

    // Test get by ID
    let retrieved_task = task_repo.get_by_id(custom_task.id).await.unwrap();
    assert!(retrieved_task.is_some());
    let retrieved_task = retrieved_task.unwrap();
    assert_eq!(retrieved_task.name, "Custom Task");
    assert_eq!(retrieved_task.max_sessions, 2);

    // Test get all (should now have 2 tasks)
    let all_tasks = task_repo.get_all().await.unwrap();
    assert_eq!(all_tasks.len(), 2);

    // Test get by tags
    let work_tasks = task_repo.get_by_tags(&["work".to_string()]).await.unwrap();
    assert_eq!(work_tasks.len(), 1);
    assert_eq!(work_tasks[0].name, "Custom Task");

    // Test update
    let mut updated_task = retrieved_task.clone();
    updated_task.name = "Updated Task".to_string();
    task_repo.update(updated_task.clone()).await.unwrap();

    let retrieved_updated = task_repo.get_by_id(custom_task.id).await.unwrap().unwrap();
    assert_eq!(retrieved_updated.name, "Updated Task");

    // Test delete
    let deleted = task_repo.delete(custom_task.id).await.unwrap();
    assert!(deleted);

    // Verify deletion
    let deleted_task = task_repo.get_by_id(custom_task.id).await.unwrap();
    assert!(deleted_task.is_none());
}

#[tokio::test]
async fn test_config_save_and_load() {
    let (_timer_manager, _task_repo, config_repo) = create_test_context();

    // Get default config
    let mut config = config_repo.get_config().unwrap();

    // Modify config
    config.default_task_config.work_duration = Duration::from_secs(30 * 60); // 30 minutes
    config.default_audio_config.volume = 0.5;
    config.app_preferences.auto_start_work_after_break = true;

    // Save config
    config_repo.save_config(&config).unwrap();

    // Load config and verify changes
    let loaded_config = config_repo.get_config().unwrap();
    assert_eq!(loaded_config.default_task_config.work_duration, Duration::from_secs(30 * 60));
    assert_eq!(loaded_config.default_audio_config.volume, 0.5);
    assert_eq!(loaded_config.app_preferences.auto_start_work_after_break, true);
}

#[tokio::test]
async fn test_timer_phase_reset() {
    let (timer_manager, task_repo, _config_repo) = create_test_context();

    // Get default task
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    // Reset current phase
    timer_manager.reset_current_phase(Some(default_task)).await;

    let state = timer_manager.get_state().await;
    assert_eq!(state.phase, Phase::Work);
    assert_eq!(state.remaining_seconds, 25 * 60); // Should reset to work duration
}

#[tokio::test]
async fn test_timer_phase_skipping() {
    let (timer_manager, task_repo, _config_repo) = create_test_context();

    // Get default task
    let tasks = task_repo.get_all().await.unwrap();
    let default_task = &tasks[0];

    // Skip to next phase
    let (old_phase, new_phase) = timer_manager.skip_to_next_phase(Some(default_task)).await;

    assert_eq!(old_phase, Phase::Work);
    assert_eq!(new_phase, Phase::ShortBreak);

    let state = timer_manager.get_state().await;
    assert_eq!(state.phase, Phase::ShortBreak);
}

#[tokio::test]
async fn test_task_session_completion() {
    let (_timer_manager, task_repo, _config_repo) = create_test_context();

    // Create a task with 2 max sessions
    let mut task = pomotoro_lib::task::models::Task::new("Limited Task".to_string(), 2);
    task_repo.create(task.clone()).await.unwrap();

    // Complete first session
    task.increment_session();
    task_repo.update(task.clone()).await.unwrap();

    let updated_task = task_repo.get_by_id(task.id).await.unwrap().unwrap();
    assert_eq!(updated_task.current_sessions, 1);
    assert_eq!(updated_task.status, TaskStatus::Queued); // New tasks start as Queued

    // Complete second session
    task.increment_session();
    task_repo.update(task.clone()).await.unwrap();

    let completed_task = task_repo.get_by_id(task.id).await.unwrap().unwrap();
    assert_eq!(completed_task.current_sessions, 2);
    assert_eq!(completed_task.status, TaskStatus::Completed);
}

#[test]
fn test_basic_types_and_enums() {
    use pomotoro_lib::timer::models::{Phase, TimerStatus};
    use pomotoro_lib::task::models::TaskStatus;

    // Test Phase enum
    assert_eq!(Phase::Work, Phase::Work);
    assert_ne!(Phase::Work, Phase::ShortBreak);

    // Test TimerStatus enum
    assert_eq!(TimerStatus::Stopped, TimerStatus::Stopped);
    assert_ne!(TimerStatus::Running, TimerStatus::Paused);

    // Test TaskStatus enum
    assert_eq!(TaskStatus::Active, TaskStatus::Active);
    assert_ne!(TaskStatus::Active, TaskStatus::Completed);
}