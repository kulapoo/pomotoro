use crate::task::models::{TaskBuilder, TaskTestRepository};
use crate::timer::models::TimerTestContext;
use domain::{TaskStatus, TaskRepository};
use std::time::Duration;

pub struct E2ETaskConfig {
    pub timeout_duration: Duration,
    pub retry_attempts: u32,
    pub cleanup_on_failure: bool,
}

impl Default for E2ETaskConfig {
    fn default() -> Self {
        Self {
            timeout_duration: Duration::from_secs(30),
            retry_attempts: 3,
            cleanup_on_failure: true,
        }
    }
}

#[tokio::test]
async fn test_e2e_task_creation_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let test_repo = TaskTestRepository::empty();

    // Create a new task
    let new_task = TaskBuilder::new("E2E Test Task".to_string(), 3)
        .with_description("End-to-end test task".to_string())
        .with_tags(vec!["e2e".to_string(), "test".to_string()])
        .build();

    // Create the task
    test_repo.create(new_task.clone()).await?;

    // Verify task was created
    let retrieved_task = test_repo.get_by_id(new_task.id).await?;
    assert!(retrieved_task.is_some());

    let task = retrieved_task.unwrap();
    assert_eq!(task.name, "E2E Test Task");
    assert_eq!(task.max_sessions, 3);
    assert_eq!(task.current_sessions, 0);
    assert_eq!(task.status, TaskStatus::Queued);

    Ok(())
}

#[tokio::test]
async fn test_e2e_task_completion_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let test_repo = TaskTestRepository::empty();

    // Create a task with 2 sessions
    let mut task = TaskBuilder::new("Completion Test".to_string(), 2)
        .with_tags(vec!["completion".to_string()])
        .build();

    test_repo.create(task.clone()).await?;

    // Complete first session
    task.increment_session()?;
    test_repo.update(task.clone()).await?;

    let updated_task = test_repo.get_by_id(task.id).await?.unwrap();
    assert_eq!(updated_task.current_sessions, 1);
    assert_eq!(updated_task.status, TaskStatus::Queued);

    // Complete second session
    task.increment_session()?;
    test_repo.update(task.clone()).await?;

    let completed_task = test_repo.get_by_id(task.id).await?.unwrap();
    assert_eq!(completed_task.current_sessions, 2);
    assert_eq!(completed_task.status, TaskStatus::Completed);

    Ok(())
}

#[tokio::test]
async fn test_e2e_task_selection_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let test_repo = TaskTestRepository::empty();

    // Create multiple tasks
    let work_task = TaskBuilder::new("Work Task".to_string(), 4)
        .with_tags(vec!["work".to_string()])
        .with_status(TaskStatus::Active)
        .build();

    let study_task = TaskBuilder::new("Study Task".to_string(), 3)
        .with_tags(vec!["study".to_string()])
        .with_status(TaskStatus::Queued)
        .build();

    let personal_task = TaskBuilder::new("Personal Task".to_string(), 2)
        .with_tags(vec!["personal".to_string()])
        .with_status(TaskStatus::Queued)
        .build();

    test_repo.create(work_task.clone()).await?;
    test_repo.create(study_task.clone()).await?;
    test_repo.create(personal_task.clone()).await?;

    // Test filtering by different criteria
    let all_tasks = test_repo.get_all().await?;
    assert_eq!(all_tasks.len(), 3);

    let work_tasks = test_repo.get_by_tags(&["work".to_string()]).await?;
    assert_eq!(work_tasks.len(), 1);
    assert_eq!(work_tasks[0].name, "Work Task");

    let active_tasks: Vec<_> = all_tasks.iter()
        .filter(|t| t.status == TaskStatus::Active)
        .collect();
    assert_eq!(active_tasks.len(), 1);
    assert_eq!(active_tasks[0].name, "Work Task");

    Ok(())
}

#[tokio::test]
async fn test_e2e_task_switching_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let timer_context = TimerTestContext::new();

    // Create tasks
    let work_task = TaskBuilder::new("Work Project".to_string(), 4)
        .with_tags(vec!["work".to_string()])
        .build();

    let study_task = TaskBuilder::new("Study Session".to_string(), 3)
        .with_tags(vec!["study".to_string()])
        .build();

    timer_context.task_repo.create(work_task.clone()).await?;
    timer_context.task_repo.create(study_task.clone()).await?;

    // Start with work task
    timer_context.timer_service.switch_task(work_task.id, Some(&work_task)).await;
    let state1 = timer_context.timer_service.get_state().await;
    assert_eq!(state1.active_task_id, Some(work_task.id));

    // Switch to study task
    timer_context.timer_service.switch_task(study_task.id, Some(&study_task)).await;
    let state2 = timer_context.timer_service.get_state().await;
    assert_eq!(state2.active_task_id, Some(study_task.id));

    // Verify task switching preserves timer state structure
    assert_eq!(state2.phase(), state1.phase());
    assert_eq!(state2.status(), state1.status());

    Ok(())
}

#[tokio::test]
async fn test_e2e_task_performance() -> Result<(), Box<dyn std::error::Error>> {
    let test_repo = TaskTestRepository::empty();
    let start_time = std::time::Instant::now();

    // Create 100 tasks
    let mut task_ids = Vec::new();
    for i in 0..100 {
        let task = TaskBuilder::new(format!("Performance Task {i}"), 1)
            .with_tags(vec!["performance".to_string(), format!("batch_{}", i / 10)])
            .build();

        test_repo.create(task.clone()).await?;
        task_ids.push(task.id);
    }

    let creation_time = start_time.elapsed();
    assert!(creation_time < Duration::from_millis(1000), "Task creation took too long: {creation_time:?}");

    // Retrieve all tasks
    let retrieval_start = std::time::Instant::now();
    let all_tasks = test_repo.get_all().await?;
    let retrieval_time = retrieval_start.elapsed();

    assert_eq!(all_tasks.len(), 100);
    assert!(retrieval_time < Duration::from_millis(100), "Task retrieval took too long: {retrieval_time:?}");

    // Test filtering performance
    let filter_start = std::time::Instant::now();
    let performance_tasks = test_repo.get_by_tags(&["performance".to_string()]).await?;
    let filter_time = filter_start.elapsed();

    assert_eq!(performance_tasks.len(), 100);
    assert!(filter_time < Duration::from_millis(100), "Task filtering took too long: {filter_time:?}");

    Ok(())
}