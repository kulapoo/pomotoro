use tauri::State;
use super::models::Task;
use super::repository::TaskRepository;
use pomotoro_domain::{TaskId, TaskConfig, AudioConfig};

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
    repository: State<'_, TaskRepository>,
) -> Result<Task, String> {
    let mut task = Task::new(request.name, request.max_sessions)
        .map_err(|e| e.to_string())?;
    
    if let Some(description) = request.description {
        task = task.with_description(description);
    }
    
    if !request.tags.is_empty() {
        task = task.with_tags(request.tags);
    }
    
    if let Some(config) = request.config {
        task = task.with_config(config)
            .map_err(|e| e.to_string())?;
    }
    
    if let Some(audio_config) = request.audio_config {
        task = task.with_audio_config(audio_config)
            .map_err(|e| e.to_string())?;
    }

    repository.create(task.clone()).await.map_err(|e| e.to_string())?;
    Ok(task)
}

#[tauri::command]
pub async fn get_task(
    id: TaskId,
    repository: State<'_, TaskRepository>,
) -> Result<Option<Task>, String> {
    repository.get_by_id(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_tasks(
    repository: State<'_, TaskRepository>,
) -> Result<Vec<Task>, String> {
    repository.get_all().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_tasks(
    repository: State<'_, TaskRepository>,
) -> Result<Vec<Task>, String> {
    repository.get_active_tasks().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task(
    request: UpdateTaskRequest,
    repository: State<'_, TaskRepository>,
) -> Result<Task, String> {
    let mut task = repository
        .get_by_id(request.id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Task not found")?;

    if let Some(name) = request.name {
        task.name = name;
    }
    
    if let Some(description) = request.description {
        task.description = Some(description);
    }
    
    if let Some(max_sessions) = request.max_sessions {
        task.max_sessions = max_sessions;
    }
    
    if let Some(tags) = request.tags {
        task.tags = tags;
    }
    
    if let Some(config) = request.config {
        task.config = config;
    }
    
    if let Some(audio_config) = request.audio_config {
        task.audio_config = audio_config;
    }

    repository.update(task.clone()).await.map_err(|e| e.to_string())?;
    Ok(task)
}

#[tauri::command]
pub async fn delete_task(
    id: TaskId,
    repository: State<'_, TaskRepository>,
) -> Result<bool, String> {
    repository.delete(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tasks_by_tags(
    tags: Vec<String>,
    repository: State<'_, TaskRepository>,
) -> Result<Vec<Task>, String> {
    repository.get_by_tags(&tags).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_task_session(
    id: TaskId,
    repository: State<'_, TaskRepository>,
) -> Result<Task, String> {
    let mut task = repository
        .get_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Task not found")?;

    task.increment_session()
        .map_err(|e| e.to_string())?;
    repository.update(task.clone()).await.map_err(|e| e.to_string())?;
    Ok(task)
}

#[tauri::command]
pub async fn reset_task_sessions(
    id: TaskId,
    repository: State<'_, TaskRepository>,
) -> Result<Task, String> {
    let mut task = repository
        .get_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Task not found")?;

    task.reset_sessions();
    repository.update(task.clone()).await.map_err(|e| e.to_string())?;
    Ok(task)
}