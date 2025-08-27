use crate::adapters::{
    TaskRepositoryArc, events::mem_event_bus::EventPublisherArc,
};
use domain::{AudioConfig, Task, TaskId};
use tauri::State;
use usecases::task::*;
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
    pub audio_config: Option<AudioConfig>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateTaskRequest {
    pub id: TaskId,
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

#[tauri::command]
pub async fn create_task(
    request: CreateTaskRequest,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Task, String> {
    println!("Creating task with name: {}", request.name);

    let cmd = CreateTaskCmd {
        name: request.name.clone(),
        description: request.description,
        max_sessions: request.max_sessions,
        tags: request.tags,
    };

    usecases::task::create_task(&task_repo, &event_publisher, cmd)
        .await
        .with_context(|| format!("Failed to create task: {}", request.name))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_task(
    id: String,
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Option<Task>, String> {
    let query = GetTaskQuery { id: id.clone() };
    let result = usecases::task::get_task(&task_repo, query)
        .await
        .with_context(|| format!("Failed to get task with id: {}", id))
        .map_err(|e| e.to_string())?;
    Ok(Some(result))
}

#[tauri::command]
pub async fn get_all_tasks(
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Vec<Task>, String> {
    let query = GetTasksQuery {
        tags: None,
        status: None,
        active_only: false,
    };
    usecases::task::get_tasks(&task_repo, query)
        .await
        .context("Failed to get all tasks")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_tasks(
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Vec<Task>, String> {
    task_repo
        .get_active_tasks()
        .await
        .context("Failed to get active tasks")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task(
    request: UpdateTaskRequest,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Task, String> {
    let cmd = UpdateTaskCmd {
        id: request.id.to_string(),
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

    usecases::task::update_task(&task_repo, &event_publisher, cmd)
        .await
        .with_context(|| format!("Failed to update task with id: {}", request.id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_task(
    id: String,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<bool, String> {
    let cmd = DeleteTaskCmd { id: id.clone() };
    usecases::task::delete_task(&task_repo, &event_publisher, cmd)
        .await
        .with_context(|| format!("Failed to delete task with id: {}", id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tasks_by_tags(
    tags: Vec<String>,
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Vec<Task>, String> {
    task_repo
        .get_by_tags(&tags)
        .await
        .with_context(|| format!("Failed to get tasks with tags: {:?}", tags))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_task_session(
    task_id: String,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Task, String> {
    let _result = usecases::task::complete_session(
        &task_repo,
        &event_publisher,
        &task_id,
    )
    .await
    .with_context(|| format!("Failed to complete session for task: {}", task_id))
    .map_err(|e| e.to_string())?;

    let task_id = domain::TaskId::from_string(&task_id)
        .map_err(|_| anyhow!("Invalid task ID: {}", task_id))
        .map_err(|e| e.to_string())?;

    task_repo
        .get_by_id(task_id)
        .await
        .context("Failed to retrieve task after session completion")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| anyhow!("Task not found after completing session"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_task_sessions(
    task_id: String,
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Task, String> {
    usecases::task::reset_sessions(&task_repo, &task_id)
        .await
        .with_context(|| format!("Failed to reset sessions for task: {}", task_id))
        .map_err(|e| e.to_string())?;

    let task_id = domain::TaskId::from_string(&task_id)
        .map_err(|_| anyhow!("Invalid task ID: {}", task_id))
        .map_err(|e| e.to_string())?;

    task_repo
        .get_by_id(task_id)
        .await
        .context("Failed to retrieve task after resetting sessions")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| anyhow!("Task not found after resetting sessions"))
        .map_err(|e| e.to_string())
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

#[tauri::command]
pub async fn search_tasks(
    request: SearchTasksRequest,
    task_repo: State<'_, TaskRepositoryArc>,
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

#[tauri::command]
pub async fn search_tasks_fuzzy(
    query: String,
    task_repo: State<'_, TaskRepositoryArc>,
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

#[tauri::command]
pub async fn filter_tasks_by_status(
    request: FilterTasksRequest,
    task_repo: State<'_, TaskRepositoryArc>,
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

#[tauri::command]
pub async fn cycle_incomplete_task(
    request: CycleIncompleteTaskRequest,
    cycling_service: State<'_, Arc<dyn domain::TaskCyclerService + Send + Sync>>,
) -> Result<CycleIncompleteTaskResponse, String> {
    use usecases::task::cycle_incomplete_task::{CycleDirection, CycleIncompleteTaskQuery};
    
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

#[tauri::command]
pub async fn get_task_cycle_position(
    task_id: String,
    cycling_service: State<'_, Arc<dyn domain::TaskCyclerService + Send + Sync>>,
) -> Result<(usize, usize), String> {
    usecases::task::get_task_cycle_position(&cycling_service, task_id)
        .await
        .context("Failed to get task cycle position")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_incomplete_tasks(
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Vec<Task>, String> {
    task_repo
        .get_incomplete_tasks()
        .await
        .context("Failed to get incomplete tasks")
        .map_err(|e| e.to_string())
}
