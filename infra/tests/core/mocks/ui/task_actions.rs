use serde_json::{json, Value};
use domain::event_names::commands::task as task_commands;
use super::app_handle::MockAppHandle;

/// Task-specific UI actions
#[derive(Clone)]
pub struct TaskUiActions {
    app_handle: MockAppHandle,
}

impl TaskUiActions {
    pub fn new(app_handle: MockAppHandle) -> Self {
        Self { app_handle }
    }

    /// Create a new task
    pub async fn create_task(&self, title: &str, description: Option<&str>) -> Value {
        let payload = json!({
            "title": title,
            "description": description,
            "estimated_sessions": 1,
            "priority": "Medium"
        });

        self.app_handle.emit(task_commands::CREATE, payload).unwrap();

        json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "title": title,
            "description": description,
            "created_at": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Update an existing task
    pub async fn update_task(&self, task_id: &str, updates: Value) -> Value {
        let mut payload = updates;
        payload["id"] = json!(task_id);

        self.app_handle.emit(task_commands::UPDATE, payload).unwrap();

        json!({
            "id": task_id,
            "updated": true,
            "updated_at": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Delete a task
    pub async fn delete_task(&self, task_id: &str) -> Value {
        self.app_handle.emit(task_commands::DELETE, json!({
            "id": task_id
        })).unwrap();

        json!({
            "id": task_id,
            "deleted": true
        })
    }

    /// Complete a task session
    pub async fn complete_session(&self, task_id: &str) -> Value {
        self.app_handle.emit(task_commands::COMPLETE_SESSION, json!({
            "task_id": task_id,
            "session_type": "Work"
        })).unwrap();

        json!({
            "task_id": task_id,
            "sessions_completed": 1,
            "completed_at": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Get all tasks
    pub async fn get_all_tasks(&self) -> Value {
        self.app_handle.emit(task_commands::GET_ALL, json!({})).unwrap();

        json!([
            {
                "id": "task_1",
                "title": "Test Task 1",
                "status": "NotStarted"
            },
            {
                "id": "task_2",
                "title": "Test Task 2",
                "status": "InProgress"
            }
        ])
    }

    /// Search tasks
    pub async fn search_tasks(&self, query: &str) -> Value {
        self.app_handle.emit(task_commands::SEARCH, json!({
            "query": query
        })).unwrap();

        json!([])
    }

    /// Cycle through incomplete tasks
    pub async fn cycle_incomplete_task(&self) -> Value {
        self.app_handle.emit(task_commands::CYCLE_INCOMPLETE_TASK, json!({})).unwrap();

        json!({
            "next_task_id": "task_2",
            "position": 1,
            "total": 3
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_ui_actions() {
        let app_handle = MockAppHandle::new();
        let task_actions = TaskUiActions::new(app_handle.clone());

        // Create a task
        let task = task_actions.create_task("Test Task", Some("Test Description")).await;
        assert!(task["id"].is_string());
        assert_eq!(task["title"], "Test Task");
        assert!(app_handle.was_event_emitted(task_commands::CREATE));

        // Update the task
        let task_id = task["id"].as_str().unwrap();
        let update_result = task_actions.update_task(task_id, json!({
            "title": "Updated Task"
        })).await;
        assert_eq!(update_result["updated"], true);

        // Delete the task
        let delete_result = task_actions.delete_task(task_id).await;
        assert_eq!(delete_result["deleted"], true);
    }
}