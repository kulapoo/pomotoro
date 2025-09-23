use std::any::TypeId;

use domain::{
    TaskCreated, TaskRepository, TaskUpdated, event_names, task::TaskDeleted,
};
use usecases::{
    CreateTaskCmd, DeleteTaskCmd, UpdateTaskCmd, create_task, delete_task,
    update_task,
};

use crate::{
    TaskFixtures,
    utils::{assert_utils, setup::setup_ctx},
};

#[tokio::test]
async fn task_should_create_with_name() {
    let ctx = setup_ctx("task_should_create_with_name").await;

    let task = TaskFixtures::simple("Test Task");

    let result = create_task(
        ctx.task_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: task.name,
            description: task.description,
            max_sessions: task.max_sessions,
            tags: task.tags,
            config: None,
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert!(result.is_ok());

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::task::TASK_CREATED,
    );
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TaskCreated>());
}

#[tokio::test]
async fn task_should_have_unique_ids() {
    let ctx = setup_ctx("task_should_have_unique_ids").await;

    let task1 = TaskFixtures::simple("Test Task 1");
    let task2 = TaskFixtures::simple("Test Task 2");

    let result1 = create_task(
        ctx.task_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: task1.name,
            description: task1.description,
            max_sessions: task1.max_sessions,
            tags: task1.tags,
            config: None,
        },
    )
    .await;

    let result2 = create_task(
        ctx.task_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: task2.name,
            description: task2.description,
            max_sessions: task2.max_sessions,
            tags: task2.tags,
            config: None,
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert!(result1.clone().is_ok());
    assert!(result2.clone().is_ok());

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::task::TASK_CREATED,
    );
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TaskCreated>());

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::task::TASK_CREATED,
    );
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TaskCreated>());

    assert_ne!(&result1.clone().unwrap().id, &result2.clone().unwrap().id);

    assert_ne!(
        result1.clone().unwrap().id.inner(),
        result2.clone().unwrap().id.inner()
    );
}

#[tokio::test]
async fn task_should_find_task_by_id() {
    let ctx = setup_ctx("task_should_find_task_by_id").await;

    let fixture = TaskFixtures::simple("Test Task");

    // Create task from fixture data
    let created_task = create_task(
        ctx.task_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: fixture.name.clone(),
            description: fixture.description.clone(),
            max_sessions: fixture.max_sessions,
            tags: fixture.tags.clone(),
            config: None,
        },
    )
    .await
    .expect("Failed to create task");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify events were emitted
    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::task::TASK_CREATED,
    );
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TaskCreated>());

    // Retrieve the task by its ID
    let retrieved_task = ctx
        .task_repo
        .get_by_id(created_task.id())
        .await
        .expect("Failed to retrieve task")
        .expect("Task not found");

    // Verify the retrieved task matches both the created result and original fixture
    assert_eq!(retrieved_task.id, created_task.id());
    assert_eq!(retrieved_task.id, created_task.id);
    assert_eq!(retrieved_task.name, fixture.name);
    assert_eq!(retrieved_task.description, fixture.description);
    assert_eq!(retrieved_task.max_sessions, fixture.max_sessions);
    assert_eq!(retrieved_task.tags, fixture.tags);
    assert_eq!(retrieved_task.status, fixture.status);
}

#[tokio::test]
async fn task_should_update_task() {
    let ctx = setup_ctx("task_should_update_task").await;

    let fixture = TaskFixtures::simple("Test Task");

    let created_task = create_task(
        ctx.task_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: fixture.name,
            description: fixture.description,
            max_sessions: fixture.max_sessions,
            tags: fixture.tags,
            config: None,
        },
    )
    .await
    .expect("Failed to create task");

    let updated_task = update_task(
        ctx.task_repo.clone(),
        ctx.event_bus.clone(),
        UpdateTaskCmd {
            id: created_task.id().to_string(),
            name: Some("Updated Task".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("Failed to update task");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::task::TASK_UPDATED,
    );
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TaskUpdated>());

    assert_eq!(updated_task.name, "Updated Task");
}

#[tokio::test]
async fn task_should_delete_task() {
    let ctx = setup_ctx("task_should_delete_task").await;

    let fixture = TaskFixtures::simple("Test Task");

    let created_task = create_task(
        ctx.task_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: fixture.name,
            description: fixture.description,
            max_sessions: fixture.max_sessions,
            tags: fixture.tags,
            config: None,
        },
    )
    .await
    .expect("Failed to create task");

    let result = delete_task(
        ctx.task_repo.clone(),
        ctx.event_bus.clone(),
        DeleteTaskCmd {
            id: created_task.id().to_string(),
        },
    )
    .await
    .expect("Failed to delete task");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert_eq!(result, true);
    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::task::TASK_DELETED,
    );
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<TaskDeleted>());
}
