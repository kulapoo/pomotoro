use crate::task::models::{TaskBuilder, TaskTestRepository};
use pomotoro_domain::{TaskStatus, TaskRepository};

#[tokio::test]
async fn test_task_repository_default_task() {
    let test_repo = TaskTestRepository::with_default_task();
    let tasks = test_repo.get_all().await.unwrap();

    assert_eq!(tasks.len(), 1);

    let default_task = &tasks[0];
    assert_eq!(default_task.name, "Focus Session");
    assert_eq!(default_task.status, TaskStatus::Active);
    assert_eq!(default_task.max_sessions, 4);
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
    let work_tasks = test_repo.get_by_tags(&["work".to_string()]).await.unwrap();
    assert_eq!(work_tasks.len(), 1);
    assert_eq!(work_tasks[0].name, "Custom Task");

    // Test update
    let mut updated_task = retrieved_task.clone();
    updated_task.name = "Updated Task".to_string();
    test_repo.update(updated_task.clone()).await.unwrap();

    let retrieved_updated = test_repo.get_by_id(custom_task.id).await.unwrap().unwrap();
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

    let mut task = TaskBuilder::new("Limited Task".to_string(), 2)
        .build();
    
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

    let active_tasks: Vec<_> = all_tasks.iter()
        .filter(|t| t.status == TaskStatus::Active)
        .collect();
    assert_eq!(active_tasks.len(), 1);
    assert_eq!(active_tasks[0].name, "Active Task");

    let completed_tasks: Vec<_> = all_tasks.iter()
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
    let work_tasks = test_repo.get_by_tags(&["work".to_string()]).await.unwrap();
    assert_eq!(work_tasks.len(), 2);

    let study_tags = test_repo.get_by_tags(&["study".to_string()]).await.unwrap();
    assert_eq!(study_tags.len(), 1);

    let learning_tasks = test_repo.get_by_tags(&["learning".to_string()]).await.unwrap();
    assert_eq!(learning_tasks.len(), 2);

    // Test multiple tag filtering (should find tasks that have ANY of the tags)
    let multiple_tags = test_repo.get_by_tags(&["urgent".to_string(), "learning".to_string()]).await.unwrap();
    assert_eq!(multiple_tags.len(), 3); // All tasks have at least one of these tags
}

#[tokio::test]
async fn test_task_session_limits() {
    let test_repo = TaskTestRepository::empty();

    let mut task = TaskBuilder::new("Limited Task".to_string(), 1)
        .build();

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