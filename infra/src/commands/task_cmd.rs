use crate::adapters::{
    events::mem_event_bus::EventPublisherArc, task::task_dto::TaskDto,
};
use anyhow::{Context, anyhow};
use domain::{
    AudioConfig, Config, ConfigRepository, Task, TaskId, TaskRepository,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::State;
use log::{debug, info};
use usecases::task::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
    pub config: Option<Config>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateTaskRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub work_duration: Option<Duration>,
    pub short_break_duration: Option<Duration>,
    pub long_break_duration: Option<Duration>,
    pub sessions_until_long_break: Option<u8>,
    pub enable_screen_blocking: Option<bool>,
    pub audio_config: Option<AudioConfig>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_task(
    request: CreateTaskRequest,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<TaskDto, String> {
    debug!("Creating task: name='{}', sessions={}, tags={:?}",
           request.name, request.max_sessions, request.tags);

    let cmd = CreateTaskCmd {
        name: request.name.clone(),
        description: request.description,
        max_sessions: request.max_sessions,
        tags: request.tags,
        config: request.config,
    };

    match usecases::task::create_task(
        task_repo.inner().clone(),
        config_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    {
        Ok(task) => {
            info!("Created task: id={}, name='{}'", task.id, task.name);
            Ok(TaskDto::from(task))
        }
        Err(e) => {
            log::error!("Failed to create task '{}': {}", request.name, e);
            Err(format!("Failed to create task '{}': {}", request.name, e))
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_task(
    id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Option<TaskDto>, String> {
    let task_id = TaskId::from_string(&id)
        .map_err(|_| format!("Invalid task ID: {}", id))?;

    let result = usecases::task::get_task_by_id(&task_repo, task_id)
        .await
        .with_context(|| format!("Failed to get task with id: {}", id))
        .map_err(|e| e.to_string())?;
    Ok(Some(TaskDto::from(result)))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_all_tasks(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<TaskDto>, String> {
    let query = GetTasksQuery {
        tags: None,
        status: None,
        active_only: false,
    };
    let tasks = usecases::task::get_tasks(&task_repo, query)
        .await
        .context("Failed to get all tasks")
        .map_err(|e| e.to_string())?;
    Ok(tasks.into_iter().map(TaskDto::from).collect())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_active_tasks(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<TaskDto>, String> {
    let tasks = task_repo
        .get_active_tasks()
        .await
        .context("Failed to get active tasks")
        .map_err(|e| e.to_string())?;
    Ok(tasks.into_iter().map(TaskDto::from).collect())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_task(
    request: UpdateTaskRequest,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<TaskDto, String> {
    let task_id = TaskId::from_string(&request.id)
        .map_err(|_| format!("Invalid task ID: {}", request.id))?;

    let cmd = UpdateTaskCmd {
        id: task_id,
        name: request.name,
        description: request.description,
        max_sessions: request.max_sessions,
        tags: request.tags,
        work_duration: request.work_duration,
        short_break_duration: request.short_break_duration,
        long_break_duration: request.long_break_duration,
        sessions_until_long_break: request.sessions_until_long_break,
        enable_screen_blocking: request.enable_screen_blocking,
        audio_config: request.audio_config,
    };

    let task = usecases::task::update_task(
        task_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .map_err(|e| {
        let error_msg = format!("Failed to update task with id {}: {:?}", request.id, e);
        log::error!("{}", error_msg);
        error_msg
    })?;
    Ok(TaskDto::from(task))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_task(
    id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<bool, String> {
    info!("Deleting tae tae task: id={}", id);

    let task_id = TaskId::from_string(&id)
        .map_err(|_| format!("Invalid task ID: {}", id))?;

    let cmd = DeleteTaskCmd { id: task_id };
    usecases::task::delete_task(
        task_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .with_context(|| format!("Failed to delete task with id: {}", id))
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_tasks_by_tags(
    tags: Vec<String>,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    task_repo
        .get_by_tags(&tags)
        .await
        .with_context(|| format!("Failed to get tasks with tags: {:?}", tags))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_task_sessions(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Task, String> {
    info!("Resetting sessions for task: id={}", task_id);

    let task_id_parsed = domain::TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    usecases::task::reset_sessions(&task_repo, task_id_parsed)
        .await
        .with_context(|| {
            format!("Failed to reset sessions for task: {}", task_id)
        })
        .map_err(|e| {
            log::error!("Failed to reset sessions for task {}: {}", task_id, e);
            e.to_string()
        })?;

    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("Failed to retrieve task after resetting sessions")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| anyhow!("Task not found after resetting sessions"))
        .map_err(|e| e.to_string())?;

    info!("Successfully reset sessions for task: id={}", task_id);
    Ok(task)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_task_status(
    task_id: String,
    reset_sessions: bool,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<TaskDto, String> {
    info!("Resetting task status: id={}, reset_sessions={}", task_id, reset_sessions);

    let task_id_parsed = domain::TaskId::from_string(&task_id)
        .map_err(|e| {
            log::error!("Invalid task ID '{}': {}", task_id, e);
            format!("Invalid task ID: {}", task_id)
        })?;

    usecases::task::reset_task_status(&task_repo, task_id_parsed, reset_sessions)
        .await
        .with_context(|| {
            format!("Failed to reset status for task: {}", task_id)
        })
        .map_err(|e| {
            log::error!("Failed to reset status for task {}: {}", task_id, e);
            e.to_string()
        })?;

    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("Failed to retrieve task after resetting status")
        .map_err(|e| {
            log::error!("Failed to retrieve task {} after reset: {}", task_id, e);
            e.to_string()
        })?
        .ok_or_else(|| {
            log::error!("Task not found after resetting status: {}", task_id);
            anyhow!("Task not found after resetting status")
        })
        .map_err(|e| e.to_string())?;

    info!("Successfully reset task status: id={}, new_status={:?}", task_id, task.status);
    Ok(TaskDto::from(task))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTasksRequest {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_tasks(
    request: SearchTasksRequest,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    let query = SearchTasksQuery {
        query: request.query,
        tags: request.tags,
        status: request.status,
        sort_by: request.sort_by,
        sort_order: request.sort_order,
        limit: request.limit,
        offset: request.offset,
    };

    usecases::task::search_tasks(&task_repo, query)
        .await
        .context("Failed to search tasks")
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_tasks_fuzzy(
    query: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    usecases::task::search_tasks_fuzzy(&task_repo, query)
        .await
        .context("Failed to perform fuzzy search")
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterTasksRequest {
    pub status: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn filter_tasks_by_status(
    request: FilterTasksRequest,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    let query = FilterTasksByStatusQuery {
        status: request.status,
        limit: request.limit,
        offset: request.offset,
    };

    usecases::task::filter_tasks_by_status(&task_repo, query)
        .await
        .context("Failed to filter tasks by status")
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleIncompleteTaskRequest {
    pub current_task_id: Option<String>,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleIncompleteTaskResponse {
    pub task: Option<Task>,
    pub position: usize,
    pub total_incomplete: usize,
    pub has_more_tasks: bool,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn cycle_incomplete_task(
    request: CycleIncompleteTaskRequest,
    cycling_service: State<
        '_,
        Arc<dyn domain::TaskCyclerService + Send + Sync>,
    >,
) -> Result<CycleIncompleteTaskResponse, String> {
    use usecases::task::cycle_incomplete_task::{
        CycleDirection, CycleIncompleteTaskQuery,
    };

    let direction = match request.direction.as_str() {
        "next" => CycleDirection::Next,
        "previous" => CycleDirection::Previous,
        _ => CycleDirection::Next,
    };

    let query = CycleIncompleteTaskQuery {
        current_task_id: request.current_task_id,
        direction,
    };

    let result = usecases::task::cycle_incomplete_task(&cycling_service, query)
        .await
        .context("Failed to cycle incomplete task")
        .map_err(|e| e.to_string())?;

    Ok(CycleIncompleteTaskResponse {
        task: result.task,
        position: result.position,
        total_incomplete: result.total_incomplete,
        has_more_tasks: result.has_more_tasks,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_task_cycle_position(
    task_id: String,
    cycling_service: State<
        '_,
        Arc<dyn domain::TaskCyclerService + Send + Sync>,
    >,
) -> Result<(usize, usize), String> {
    usecases::task::get_task_cycle_position(&cycling_service, task_id)
        .await
        .context("Failed to get task cycle position")
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_incomplete_tasks(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    task_repo
        .get_incomplete_tasks()
        .await
        .context("Failed to get incomplete tasks")
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn debug_create_test_task(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<TaskDto, String> {
    println!("=== DEBUG TEST TASK CREATION ===");

    let request = CreateTaskRequest {
        name: "Debug Test Task".to_string(),
        description: Some("A test task for debugging".to_string()),
        max_sessions: 3,
        tags: vec!["debug".to_string(), "test".to_string()],
        config: None,
    };

    create_task(request, task_repo, config_repo, event_publisher).await
}
