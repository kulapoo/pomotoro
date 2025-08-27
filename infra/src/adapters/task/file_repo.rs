use super::task_dto::TaskDto;
use async_trait::async_trait;
use domain::{
    Error, Result, Task, TaskId, TaskRepository, TaskStatus,
    shared_kernel::traits::searchable::{SearchCriteria, Searchable},
    task::repository::{SearchOptions, SortBy, SortOrder},
};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// File-based task repository that demonstrates proper DTO usage
/// This shows how infrastructure layer should handle serialization
/// while keeping domain objects pure
pub struct FileTaskRepository {
    tasks_file: PathBuf,
}

impl FileTaskRepository {
    pub fn new(tasks_file: PathBuf) -> Self {
        Self { tasks_file }
    }

    /// Load tasks from file using DTOs for deserialization
    fn load_tasks(&self) -> Result<HashMap<TaskId, Task>> {
        if !self.tasks_file.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.tasks_file).map_err(|e| {
            Error::RepositoryError {
                message: format!("Failed to read tasks file: {e}"),
            }
        })?;

        let task_dtos: Vec<TaskDto> =
            serde_json::from_str(&content).map_err(|e| {
                Error::RepositoryError {
                    message: format!("Failed to deserialize tasks: {e}"),
                }
            })?;

        let mut tasks = HashMap::new();
        for dto in task_dtos {
            let task =
                Task::try_from(dto).map_err(|e| Error::RepositoryError {
                    message: format!("Failed to convert DTO to Task: {e:?}"),
                })?;
            tasks.insert(task.id, task);
        }

        Ok(tasks)
    }

    /// Save tasks to file using DTOs for serialization
    fn save_tasks(&self, tasks: &HashMap<TaskId, Task>) -> Result<()> {
        let task_dtos: Vec<TaskDto> = tasks
            .values()
            .map(|task| TaskDto::from(task.clone()))
            .collect();

        let content =
            serde_json::to_string_pretty(&task_dtos).map_err(|e| {
                Error::RepositoryError {
                    message: format!("Failed to serialize tasks: {e}"),
                }
            })?;

        let mut file = fs::File::create(&self.tasks_file).map_err(|e| {
            Error::RepositoryError {
                message: format!("Failed to create tasks file: {e}"),
            }
        })?;

        file.write_all(content.as_bytes()).map_err(|e| {
            Error::RepositoryError {
                message: format!("Failed to write tasks file: {e}"),
            }
        })?;

        file.sync_all().map_err(|e| Error::RepositoryError {
            message: format!("Failed to sync tasks file: {e}"),
        })?;

        Ok(())
    }
}

#[async_trait]
impl TaskRepository for FileTaskRepository {
    async fn create(&self, task: Task) -> Result<()> {
        let mut tasks = self.load_tasks()?;

        if tasks.contains_key(&task.id) {
            return Err(Error::RepositoryError {
                message: format!("Task with ID {} already exists", task.id),
            });
        }

        tasks.insert(task.id, task);
        self.save_tasks(&tasks)?;
        Ok(())
    }

    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        let mut task_list: Vec<Task> = tasks.values().cloned().collect();
        task_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(task_list)
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks
            .values()
            .filter(|task| {
                matches!(task.status, TaskStatus::Active | TaskStatus::Queued)
            })
            .cloned()
            .collect())
    }

    async fn update(&self, task: Task) -> Result<()> {
        let mut tasks = self.load_tasks()?;

        if !tasks.contains_key(&task.id) {
            return Err(Error::TaskNotFound {
                id: task.id.to_string(),
            });
        }

        tasks.insert(task.id, task);
        self.save_tasks(&tasks)?;
        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<bool> {
        let mut tasks = self.load_tasks()?;
        let removed = tasks.remove(&id).is_some();
        if removed {
            self.save_tasks(&tasks)?;
        }
        Ok(removed)
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks
            .values()
            .filter(|task| tags.iter().any(|tag| task.tags.contains(tag)))
            .cloned()
            .collect())
    }

    async fn get_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks
            .values()
            .filter(|task| task.status == status)
            .cloned()
            .collect())
    }

    async fn exists(&self, id: TaskId) -> Result<bool> {
        let tasks = self.load_tasks()?;
        Ok(tasks.contains_key(&id))
    }

    async fn get_default_task(&self) -> Result<Option<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks.values().find(|task| task.default).cloned())
    }
    
    async fn search(&self, options: SearchOptions) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        
        let mut result: Vec<Task> = tasks.values()
            .filter(|task| {
                let mut matches = true;
                
                if let Some(ref query) = options.criteria.query {
                    let query_lower = query.to_lowercase();
                    let name_matches = task.name.to_lowercase().contains(&query_lower);
                    let description_matches = task.description
                        .as_ref()
                        .map_or(false, |desc| desc.to_lowercase().contains(&query_lower));
                    let tag_matches = task.tags.iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower));
                    matches = matches && (name_matches || description_matches || tag_matches);
                }
                
                if let Some(ref tags) = options.criteria.tags {
                    let has_any_tag = tags.iter()
                        .any(|tag| task.tags.contains(tag));
                    matches = matches && has_any_tag;
                }
                
                if let Some(ref status_str) = options.criteria.status {
                    let status_matches = match status_str.to_lowercase().as_str() {
                        "active" => task.status == TaskStatus::Active,
                        "completed" => task.status == TaskStatus::Completed,
                        "paused" => task.status == TaskStatus::Paused,
                        "queued" => task.status == TaskStatus::Queued,
                        _ => false,
                    };
                    matches = matches && status_matches;
                }
                
                matches
            })
            .cloned()
            .collect();
        
        if let Some(sort_by) = options.sort_by {
            let sort_order = options.sort_order.unwrap_or(SortOrder::Ascending);
            result.sort_by(|a, b| {
                let ordering = match sort_by {
                    SortBy::Name => a.name.cmp(&b.name),
                    SortBy::CreatedAt => a.created_at.cmp(&b.created_at),
                    SortBy::SessionsCompleted => a.current_sessions.cmp(&b.current_sessions),
                    SortBy::Status => {
                        let status_order = |s: &TaskStatus| match s {
                            TaskStatus::Active => 0,
                            TaskStatus::Queued => 1,
                            TaskStatus::Paused => 2,
                            TaskStatus::Completed => 3,
                        };
                        status_order(&a.status).cmp(&status_order(&b.status))
                    },
                };
                match sort_order {
                    SortOrder::Ascending => ordering,
                    SortOrder::Descending => ordering.reverse(),
                }
            });
        }
        
        if let Some(limit) = options.criteria.limit {
            let offset = options.criteria.offset.unwrap_or(0);
            result = result.into_iter().skip(offset).take(limit).collect();
        }
        
        Ok(result)
    }
    
    async fn search_fuzzy(&self, query: &str) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        
        let query_lower = query.to_lowercase();
        let query_parts: Vec<&str> = query_lower.split_whitespace().collect();
        
        let mut scored_tasks: Vec<(Task, usize)> = tasks.values()
            .map(|task| {
                let mut score = 0;
                
                for part in &query_parts {
                    if task.name.to_lowercase().contains(part) {
                        score += 3;
                    }
                    if let Some(ref desc) = task.description {
                        if desc.to_lowercase().contains(part) {
                            score += 2;
                        }
                    }
                    for tag in &task.tags {
                        if tag.to_lowercase().contains(part) {
                            score += 1;
                        }
                    }
                }
                
                (task.clone(), score)
            })
            .filter(|(_, score)| *score > 0)
            .collect();
        
        scored_tasks.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.created_at.cmp(&b.0.created_at)));
        
        Ok(scored_tasks.into_iter().map(|(task, _)| task).collect())
    }
    
    async fn get_incomplete_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks
            .values()
            .filter(|task| !task.is_completed())
            .cloned()
            .collect())
    }
    
    async fn get_completed_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks
            .values()
            .filter(|task| task.is_completed())
            .cloned()
            .collect())
    }
}

#[async_trait]
impl Searchable<Task> for FileTaskRepository {
    async fn search(&self, criteria: &SearchCriteria) -> Result<Vec<Task>> {
        let options = SearchOptions {
            criteria: criteria.clone(),
            sort_by: None,
            sort_order: None,
        };
        TaskRepository::search(self, options).await
    }
    
    async fn search_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        self.get_by_tags(tags).await
    }
    
    async fn search_by_query(&self, query: &str) -> Result<Vec<Task>> {
        self.search_fuzzy(query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use domain::{
        shared_kernel::traits::searchable::SearchCriteria,
        task::repository::{Repository, SearchOptions, SortBy, SortOrder}
    };

    #[tokio::test]
    async fn should_save_and_load_tasks_with_dtos() {
        let temp_dir = tempdir().unwrap();
        let tasks_file = temp_dir.path().join("tasks.json");
        let repo = FileTaskRepository::new(tasks_file);

        let task = Task::new("Test Task".to_string(), 4).unwrap();
        let task_id = task.id;

        repo.create(task.clone()).await.unwrap();

        let loaded_task = repo.get_by_id(task_id).await.unwrap().unwrap();
        assert_eq!(loaded_task.name, "Test Task");
        assert_eq!(loaded_task.max_sessions, 4);

        // Verify file contains DTO format, not domain objects
        let file_content = fs::read_to_string(&repo.tasks_file).unwrap();
        assert!(file_content.contains("\"id\":"));
        assert!(file_content.contains("\"name\":"));
        // Should contain serialized TaskSettings, not TaskConfig methods
        assert!(file_content.contains("\"settings\":"));
    }

    #[tokio::test]
    async fn should_handle_dto_conversion_errors() {
        let temp_dir = tempdir().unwrap();
        let tasks_file = temp_dir.path().join("invalid_tasks.json");

        // Write invalid DTO to file
        fs::write(&tasks_file, r#"[{"id": "invalid-id", "name": ""}]"#)
            .unwrap();

        let repo = FileTaskRepository::new(tasks_file);
        let result = repo.get_all().await;

        // Should handle DTO conversion errors gracefully
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_search_tasks_by_name() {
        let temp_dir = tempdir().unwrap();
        let tasks_file = temp_dir.path().join("search_tasks.json");
        let repo = FileTaskRepository::new(tasks_file);

        let task1 = Task::new("Project Alpha".to_string(), 4).unwrap();
        let task2 = Task::new("Project Beta".to_string(), 3).unwrap();
        let task3 = Task::new("Review Documents".to_string(), 2).unwrap();

        repo.create(task1.clone()).await.unwrap();
        repo.create(task2.clone()).await.unwrap();
        repo.create(task3.clone()).await.unwrap();

        let options = SearchOptions {
            criteria: SearchCriteria::new().with_query("Project".to_string()),
            sort_by: Some(SortBy::Name),
            sort_order: Some(SortOrder::Ascending),
        };

        let results = Repository::search(&repo, options).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "Project Alpha");
        assert_eq!(results[1].name, "Project Beta");
    }

    #[tokio::test]
    async fn should_search_fuzzy() {
        let temp_dir = tempdir().unwrap();
        let tasks_file = temp_dir.path().join("fuzzy_search_tasks.json");
        let repo = FileTaskRepository::new(tasks_file);

        let task1 = Task::new("Documentation Update".to_string(), 2).unwrap();
        let mut task2 = Task::new("Code Review".to_string(), 3).unwrap();
        task2.tags.push("doc".to_string());
        let task3 = Task::new("Feature Implementation".to_string(), 4).unwrap();

        repo.create(task1.clone()).await.unwrap();
        repo.create(task2.clone()).await.unwrap();
        repo.create(task3.clone()).await.unwrap();

        let results = Repository::search_fuzzy(&repo, "doc").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn should_get_incomplete_and_completed_tasks() {
        let temp_dir = tempdir().unwrap();
        let tasks_file = temp_dir.path().join("completion_tasks.json");
        let repo = FileTaskRepository::new(tasks_file);

        let task1 = Task::new("Incomplete Task".to_string(), 4).unwrap();
        let mut task2 = Task::new("Completed Task".to_string(), 2).unwrap();
        task2.increment_session().unwrap();
        task2.increment_session().unwrap();
        assert!(task2.is_completed());
        let task3 = Task::new("Another Incomplete".to_string(), 3).unwrap();

        repo.create(task1.clone()).await.unwrap();
        repo.create(task2.clone()).await.unwrap();
        repo.create(task3.clone()).await.unwrap();

        let incomplete = repo.get_incomplete_tasks().await.unwrap();
        assert_eq!(incomplete.len(), 2);
        assert!(!incomplete.iter().any(|t| t.name == "Completed Task"));

        let completed = repo.get_completed_tasks().await.unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].name, "Completed Task");
    }

    #[tokio::test]
    async fn should_handle_default_task() {
        let temp_dir = tempdir().unwrap();
        let tasks_file = temp_dir.path().join("default_task.json");
        let repo = FileTaskRepository::new(tasks_file);

        let mut default_task = Task::new("Default Task".to_string(), 4).unwrap();
        default_task.set_as_default();
        let regular_task = Task::new("Regular Task".to_string(), 3).unwrap();

        repo.create(default_task.clone()).await.unwrap();
        repo.create(regular_task.clone()).await.unwrap();

        let found_default = repo.get_default_task().await.unwrap();
        assert!(found_default.is_some());
        assert_eq!(found_default.unwrap().name, "Default Task");
    }
}
