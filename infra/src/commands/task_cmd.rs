use tauri::State;
use domain::{Task, TaskId, TaskConfig, AudioConfig};
use usecases::task::*;
use crate::adapters::{events::mem_event_bus::EventPublisherArc, TaskRepositoryArc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
    pub config: Option<TaskConfig>,
    pub audio_config: Option<AudioConfig>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateTaskRequest {
    pub id: TaskId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub config: Option<TaskConfig>,
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
        name: request.name,
        description: request.description,
        max_sessions: request.max_sessions,
        tags: request.tags,
    };

    usecases::task::create_task(&task_repo, &event_publisher, cmd)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_task(
    id: String,
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Option<Task>, String> {
    let query = GetTaskQuery { id };
    let result = usecases::task::get_task(&task_repo, query)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Some(result))
}

#[tauri::command]
pub async fn get_all_tasks(
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Vec<Task>, String> {
    let query = GetTasksQuery { tags: None, status: None, active_only: false };
    usecases::task::get_tasks(&task_repo, query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_tasks(
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Vec<Task>, String> {
    task_repo.get_active_tasks()
        .await
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
        config: request.config,
        audio_config: request.audio_config,
    };

    usecases::task::update_task(&task_repo, &event_publisher, cmd)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_task(
    id: String,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<bool, String> {
    let cmd = DeleteTaskCmd { id };
    usecases::task::delete_task(&task_repo, &event_publisher, cmd)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tasks_by_tags(
    tags: Vec<String>,
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Vec<Task>, String> {
    task_repo.get_by_tags(&tags)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_task_session(
    task_id: String,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Task, String> {
    let _result = usecases::task::complete_session(&task_repo, &event_publisher, &task_id)
        .await
        .map_err(|e| e.to_string())?;

    // Return the updated task
    let task_id = domain::TaskId::from_string(&task_id)
        .map_err(|_| "Invalid task ID".to_string())?;

    task_repo.get_by_id(task_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Task not found".to_string())
}

#[tauri::command]
pub async fn reset_task_sessions(
    task_id: String,
    task_repo: State<'_, TaskRepositoryArc>,
) -> Result<Task, String> {
    usecases::task::reset_sessions(&task_repo, &task_id)
        .await
        .map_err(|e| e.to_string())?;

    // Return the updated task
    let task_id = domain::TaskId::from_string(&task_id)
        .map_err(|_| "Invalid task ID".to_string())?;

    task_repo.get_by_id(task_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Task not found".to_string())
}