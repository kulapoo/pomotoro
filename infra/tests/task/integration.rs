use crate::task::models::{TaskBuilder, TaskTestRepository};
use domain::{
    AudioConfig, TaskBuilder as DomainTaskBuilder, TaskRepository,
    TaskStatus,
};
use std::time::Duration;

#[tokio::test]
async fn test_task_repository_default_task() {
    let test_repo = TaskTestRepository::with_default_task();
    let tasks = test_repo.get_all().await.unwrap();

    assert_eq!(tasks.len(), 1);

    let default_task = &tasks[0];
    assert_eq!(default_task.name, "Focus Session");
    assert_eq!(default_task.status, TaskStatus::Active);
    assert_eq!(default_task.max_sessions, 4); // Traditional pomodoro cycle
    assert_eq!(default_task.current_sessions, 0);
    assert!(default_task.tags.contains(&"focus".to_string()));
}

#[tokio::test]
async fn test_task_crud_operations() {
    let test_repo = TaskTestRepository::empty();

    let custom_task = TaskBuilder::new("Custom Task".to_string(), 2)
        .with_tags(vec!["work".to_string(), "test".to_string()])
        .build();

    // Test create
    test_repo.create(custom_task.clone()).await.unwrap();

    // Test get by ID
    let retrieved_task = test_repo.get_by_id(custom_task.id).await.unwrap();
    assert!(retrieved_task.is_some());
    let retrieved_task = retrieved_task.unwrap();
    assert_eq!(retrieved_task.name, "Custom Task");
    assert_eq!(retrieved_task.max_sessions, 2);

    // Test get all (should now have 1 task)
    let all_tasks = test_repo.get_all().await.unwrap();
    assert_eq!(all_tasks.len(), 1);

    // Test get by tags
    let work_tasks =
        test_repo.get_by_tags(&["work".to_string()]).await.unwrap();
    assert_eq!(work_tasks.len(), 1);
    assert_eq!(work_tasks[0].name, "Custom Task");

    // Test update
    let mut updated_task = retrieved_task.clone();
    updated_task.name = "Updated Task".to_string();
    test_repo.update(updated_task.clone()).await.unwrap();

    let retrieved_updated =
        test_repo.get_by_id(custom_task.id).await.unwrap().unwrap();
    assert_eq!(retrieved_updated.name, "Updated Task");

    // Test delete
    let deleted = test_repo.delete(custom_task.id).await.unwrap();
    assert!(deleted);

    // Verify deletion
    let deleted_task = test_repo.get_by_id(custom_task.id).await.unwrap();
    assert!(deleted_task.is_none());
}

#[tokio::test]
async fn test_task_session_completion() {
    let test_repo = TaskTestRepository::empty();

    let mut task = TaskBuilder::new("Limited Task".to_string(), 2).build();

    test_repo.create(task.clone()).await.unwrap();

    // Complete first session
    task.increment_session().unwrap();
    test_repo.update(task.clone()).await.unwrap();

    let updated_task = test_repo.get_by_id(task.id).await.unwrap().unwrap();
    assert_eq!(updated_task.current_sessions, 1);
    assert_eq!(updated_task.status, TaskStatus::Queued);

    // Complete second session
    task.increment_session().unwrap();
    test_repo.update(task.clone()).await.unwrap();

    let completed_task = test_repo.get_by_id(task.id).await.unwrap().unwrap();
    assert_eq!(completed_task.current_sessions, 2);
    assert_eq!(completed_task.status, TaskStatus::Completed);
}

#[tokio::test]
async fn test_task_filtering_by_status() {
    let test_repo = TaskTestRepository::empty();

    // Create tasks with different statuses
    let active_task = TaskBuilder::new("Active Task".to_string(), 2)
        .with_status(TaskStatus::Active)
        .build();

    let queued_task = TaskBuilder::new("Queued Task".to_string(), 3)
        .with_status(TaskStatus::Queued)
        .build();

    let completed_task = TaskBuilder::new("Completed Task".to_string(), 1)
        .completed()
        .build();

    test_repo.create(active_task.clone()).await.unwrap();
    test_repo.create(queued_task.clone()).await.unwrap();
    test_repo.create(completed_task.clone()).await.unwrap();

    // Test filtering
    let all_tasks = test_repo.get_all().await.unwrap();
    assert_eq!(all_tasks.len(), 3);

    let active_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Active)
        .collect();
    assert_eq!(active_tasks.len(), 1);
    assert_eq!(active_tasks[0].name, "Active Task");

    let completed_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Completed)
        .collect();
    assert_eq!(completed_tasks.len(), 1);
    assert_eq!(completed_tasks[0].name, "Completed Task");
}

#[tokio::test]
async fn test_task_tag_filtering() {
    let test_repo = TaskTestRepository::empty();

    let work_task = TaskBuilder::new("Work Task".to_string(), 2)
        .with_tags(vec!["work".to_string(), "urgent".to_string()])
        .build();

    let study_task = TaskBuilder::new("Study Task".to_string(), 3)
        .with_tags(vec!["study".to_string(), "learning".to_string()])
        .build();

    let mixed_task = TaskBuilder::new("Mixed Task".to_string(), 1)
        .with_tags(vec!["work".to_string(), "learning".to_string()])
        .build();

    test_repo.create(work_task).await.unwrap();
    test_repo.create(study_task).await.unwrap();
    test_repo.create(mixed_task).await.unwrap();

    // Test single tag filtering
    let work_tasks =
        test_repo.get_by_tags(&["work".to_string()]).await.unwrap();
    assert_eq!(work_tasks.len(), 2);

    let study_tags =
        test_repo.get_by_tags(&["study".to_string()]).await.unwrap();
    assert_eq!(study_tags.len(), 1);

    let learning_tasks = test_repo
        .get_by_tags(&["learning".to_string()])
        .await
        .unwrap();
    assert_eq!(learning_tasks.len(), 2);

    // Test multiple tag filtering (should find tasks that have ANY of the tags)
    let multiple_tags = test_repo
        .get_by_tags(&["urgent".to_string(), "learning".to_string()])
        .await
        .unwrap();
    assert_eq!(multiple_tags.len(), 3); // All tasks have at least one of these tags
}

#[tokio::test]
async fn test_task_session_limits() {
    let test_repo = TaskTestRepository::empty();

    let mut task = TaskBuilder::new("Limited Task".to_string(), 1).build();

    test_repo.create(task.clone()).await.unwrap();

    // Complete the only allowed session
    task.increment_session().unwrap();
    test_repo.update(task.clone()).await.unwrap();

    let completed_task = test_repo.get_by_id(task.id).await.unwrap().unwrap();
    assert_eq!(completed_task.current_sessions, 1);
    assert_eq!(completed_task.status, TaskStatus::Completed);

    // Try to increment beyond limit (should fail)
    let result = task.increment_session();
    assert!(result.is_err());
}

// MVP 2.0 Enhanced Task Management Tests

#[tokio::test]
async fn test_default_task_non_deletable() {
    let test_repo = TaskTestRepository::with_default_task();
    let tasks = test_repo.get_all().await.unwrap();

    let default_task = &tasks[0];

    // Test that default task cannot be deleted (per MVP2 spec)
    let delete_result = test_repo.delete(default_task.id).await.unwrap();
    assert!(
        !delete_result,
        "Default 'Focus Session' task should not be deletable"
    );

    // Verify it still exists
    let remaining_tasks = test_repo.get_all().await.unwrap();
    assert_eq!(remaining_tasks.len(), 1);
    assert_eq!(remaining_tasks[0].name, "Focus Session");
}

#[tokio::test]
async fn test_predefined_tag_system() {
    let test_repo = TaskTestRepository::empty();

    // Test MVP2 predefined tags: work, personal, learning, health
    let work_task = TaskBuilder::new("Code Review".to_string(), 3)
        .with_tags(vec!["work".to_string(), "development".to_string()])
        .build();

    let personal_task = TaskBuilder::new("Meditation".to_string(), 1)
        .with_tags(vec![
            "personal".to_string(),
            "health".to_string(),
            "mindfulness".to_string(),
        ])
        .build();

    let learning_task = TaskBuilder::new("Rust Tutorial".to_string(), 4)
        .with_tags(vec!["learning".to_string(), "programming".to_string()])
        .build();

    let health_task = TaskBuilder::new("Workout".to_string(), 2)
        .with_tags(vec!["health".to_string(), "fitness".to_string()])
        .build();

    test_repo.create(work_task).await.unwrap();
    test_repo.create(personal_task).await.unwrap();
    test_repo.create(learning_task).await.unwrap();
    test_repo.create(health_task).await.unwrap();

    // Test filtering by MVP2 predefined tags
    let work_tasks =
        test_repo.get_by_tags(&["work".to_string()]).await.unwrap();
    assert_eq!(work_tasks.len(), 1);
    assert_eq!(work_tasks[0].name, "Code Review");

    let health_tasks = test_repo
        .get_by_tags(&["health".to_string()])
        .await
        .unwrap();
    assert_eq!(health_tasks.len(), 2); // Both meditation and workout

    let learning_tasks = test_repo
        .get_by_tags(&["learning".to_string()])
        .await
        .unwrap();
    assert_eq!(learning_tasks.len(), 1);
    assert_eq!(learning_tasks[0].name, "Rust Tutorial");

    let personal_tasks = test_repo
        .get_by_tags(&["personal".to_string()])
        .await
        .unwrap();
    assert_eq!(personal_tasks.len(), 1);
    assert_eq!(personal_tasks[0].name, "Meditation");
}

#[tokio::test]
async fn test_task_with_custom_configuration() {
    let test_repo = TaskTestRepository::empty();

    // Test task with custom timing configuration (MVP2 feature)
    let custom_task = DomainTaskBuilder::with_name_and_sessions(
        "Deep Focus Work".to_string(),
        6,
    )
    .with_description(
        "Long-form focused work requiring extended sessions".to_string(),
    )
    .with_tags(vec!["work".to_string(), "deep-focus".to_string()])
    .with_config(
        Duration::from_secs(45 * 60), // 45 min work sessions
        Duration::from_secs(10 * 60), // 10 min short breaks
        Duration::from_secs(25 * 60), // 25 min long breaks
        3,                            // Long break every 3 sessions
        true,                         // Enable screen blocking
    )
    .with_audio_config(AudioConfig {
        volume: 0.4,
        enable_background_audio: true,
        background_sound: Some("binaural-focus".to_string()),
        work_notification_sound: Some("subtle-chime".to_string()),
        break_notification_sound: Some("gentle-bell".to_string()),
        muted: false,
    })
    .build()
    .unwrap();

    test_repo.create(custom_task.clone()).await.unwrap();

    let retrieved = test_repo.get_by_id(custom_task.id).await.unwrap().unwrap();

    // Verify custom configuration
    assert_eq!(retrieved.name, "Deep Focus Work");
    assert_eq!(retrieved.max_sessions, 6);
    assert_eq!(
        retrieved.description,
        Some("Long-form focused work requiring extended sessions".to_string())
    );
    assert_eq!(retrieved.settings.work_duration, Some(Duration::from_secs(45 * 60)));
    assert_eq!(
        retrieved.settings.short_break_duration,
        Some(Duration::from_secs(10 * 60))
    );
    assert_eq!(
        retrieved.settings.long_break_duration,
        Some(Duration::from_secs(25 * 60))
    );
    assert_eq!(retrieved.settings.sessions_until_long_break, Some(3));
    assert_eq!(retrieved.settings.enable_screen_blocking, Some(true));

    // Verify audio configuration
    assert_eq!(retrieved.audio_config.volume, 0.4);
    assert!(retrieved.audio_config.enable_background_audio);
    assert_eq!(
        retrieved.audio_config.background_sound,
        Some("binaural-focus".to_string())
    );
}

#[tokio::test]
async fn test_independent_session_tracking() {
    let test_repo = TaskTestRepository::empty();

    // Create multiple tasks with different session limits
    let mut short_task =
        TaskBuilder::new("Quick Review".to_string(), 1).build();
    let mut medium_task =
        TaskBuilder::new("Feature Development".to_string(), 3).build();
    let mut long_task =
        TaskBuilder::new("Research Project".to_string(), 8).build();

    test_repo.create(short_task.clone()).await.unwrap();
    test_repo.create(medium_task.clone()).await.unwrap();
    test_repo.create(long_task.clone()).await.unwrap();

    // Complete short task entirely
    short_task.increment_session().unwrap();
    test_repo.update(short_task.clone()).await.unwrap();

    // Partially complete medium task
    medium_task.increment_session().unwrap();
    test_repo.update(medium_task.clone()).await.unwrap();

    // Partially complete long task
    long_task.increment_session().unwrap();
    long_task.increment_session().unwrap();
    test_repo.update(long_task.clone()).await.unwrap();

    // Verify independent tracking
    let updated_short =
        test_repo.get_by_id(short_task.id).await.unwrap().unwrap();
    let updated_medium =
        test_repo.get_by_id(medium_task.id).await.unwrap().unwrap();
    let updated_long =
        test_repo.get_by_id(long_task.id).await.unwrap().unwrap();

    assert_eq!(updated_short.current_sessions, 1);
    assert_eq!(updated_short.status, TaskStatus::Completed);
    assert_eq!(updated_short.get_progress_ratio(), 1.0);

    assert_eq!(updated_medium.current_sessions, 1);
    assert_eq!(updated_medium.status, TaskStatus::Queued);
    assert!((updated_medium.get_progress_ratio() - 0.333).abs() < 0.01);

    assert_eq!(updated_long.current_sessions, 2);
    assert_eq!(updated_long.status, TaskStatus::Queued);
    assert_eq!(updated_long.get_progress_ratio(), 0.25);
    assert_eq!(updated_long.get_remaining_sessions(), 6);
}

#[tokio::test]
async fn test_task_completion_workflow() {
    let test_repo = TaskTestRepository::empty();

    let mut task = TaskBuilder::new("Completion Test".to_string(), 3)
        .with_tags(vec!["test".to_string()])
        .build();

    test_repo.create(task.clone()).await.unwrap();

    // Verify initial state
    assert_eq!(task.status, TaskStatus::Queued);
    assert_eq!(task.get_remaining_sessions(), 3);
    assert!(!task.is_completed());
    assert!(task.completed_at.is_none());

    // Complete sessions one by one
    task.increment_session().unwrap();
    assert_eq!(task.current_sessions, 1);
    assert_eq!(task.status, TaskStatus::Queued);
    assert!(!task.is_completed());

    task.increment_session().unwrap();
    assert_eq!(task.current_sessions, 2);
    assert_eq!(task.status, TaskStatus::Queued);
    assert!(!task.is_completed());

    // Final session should auto-complete
    task.increment_session().unwrap();
    assert_eq!(task.current_sessions, 3);
    assert_eq!(task.status, TaskStatus::Completed);
    assert!(task.is_completed());
    assert!(task.completed_at.is_some());
    assert_eq!(task.get_remaining_sessions(), 0);

    // Cannot increment beyond completion
    let over_limit_result = task.increment_session();
    assert!(over_limit_result.is_err());
}

#[tokio::test]
async fn test_task_status_transitions() {
    let test_repo = TaskTestRepository::empty();

    let mut task = TaskBuilder::new("Status Test".to_string(), 2).build();
    test_repo.create(task.clone()).await.unwrap();

    // Default status should be Queued
    assert_eq!(task.status, TaskStatus::Queued);

    // Test activate
    task.activate().unwrap();
    assert_eq!(task.status, TaskStatus::Active);

    // Test pause
    task.pause().unwrap();
    assert_eq!(task.status, TaskStatus::Paused);

    // Test reactivate
    task.activate().unwrap();
    assert_eq!(task.status, TaskStatus::Active);

    // Test queue
    task.queue().unwrap();
    assert_eq!(task.status, TaskStatus::Queued);

    // Complete the task
    task.increment_session().unwrap();
    task.increment_session().unwrap();
    assert_eq!(task.status, TaskStatus::Completed);

    // Cannot change status of completed task
    assert!(task.activate().is_err());
    assert!(task.pause().is_err());
    assert!(task.queue().is_err());
}
