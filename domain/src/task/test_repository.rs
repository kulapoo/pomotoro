use super::{Task, id::Id, repository::{Repository, SearchOptions}, status::Status};
use crate::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory task repository for testing purposes
#[derive(Debug, Default)]
pub struct InMemoryRepository {
    tasks: Arc<Mutex<HashMap<Id, Task>>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_default_task() -> Self {
        let repo = Self::new();
        
        // Create default "Focus Session" task using builder
        let default_task = crate::task::Builder::with_name_and_sessions("Focus Session".to_string(), 4)
            .with_tags(vec!["focus".to_string()])
            .status(Status::Active)
            .default(true)
            .build()
            .expect("Failed to create default task");
        
        let mut tasks = repo.tasks.lock().unwrap();
        tasks.insert(default_task.id, default_task);
        drop(tasks);
        
        repo
    }
}

#[async_trait]
impl Repository for InMemoryRepository {
    async fn create(&self, task: Task) -> Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task.id, task);
        Ok(())
    }

    async fn get_by_id(&self, id: Id) -> Result<Option<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        let mut task_list: Vec<Task> = tasks.values().cloned().collect();
        task_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(task_list)
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        let mut active_tasks: Vec<Task> = tasks
            .values()
            .filter(|task| task.status != Status::Completed)
            .cloned()
            .collect();
        // Sort by creation time for consistent ordering
        active_tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(active_tasks)
    }

    async fn update(&self, task: Task) -> Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task.id, task);
        Ok(())
    }

    async fn delete(&self, id: Id) -> Result<bool> {
        let mut tasks = self.tasks.lock().unwrap();
        
        // Check if task exists and is the special "Focus Session" default
        if let Some(task) = tasks.get(&id) {
            if task.default && task.name == "Focus Session" {
                // The special "Focus Session" default task cannot be deleted
                return Ok(false);
            }
        }
        
        Ok(tasks.remove(&id).is_some())
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks
            .values()
            .filter(|task| tags.iter().any(|tag| task.tags.contains(tag)))
            .cloned()
            .collect())
    }

    async fn get_by_status(&self, status: Status) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks
            .values()
            .filter(|task| task.status == status)
            .cloned()
            .collect())
    }

    async fn exists(&self, id: Id) -> Result<bool> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks.contains_key(&id))
    }

    async fn get_default_task(&self) -> Result<Option<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks.values().find(|task| task.default).cloned())
    }

    async fn search(&self, options: SearchOptions) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        let mut results: Vec<Task> = tasks
            .values()
            .filter(|task| {
                let criteria = &options.criteria;
                let query_match = criteria.query.as_ref()
                    .map_or(true, |q| {
                        let q_lower = q.to_lowercase();
                        task.name.to_lowercase().contains(&q_lower)
                            || task.description.as_ref()
                                .map_or(false, |d| d.to_lowercase().contains(&q_lower))
                    });
                let tags_match = criteria.tags.as_ref()
                    .map_or(true, |tags| tags.iter().any(|tag| task.tags.contains(tag)));
                let status_match = criteria.status.as_ref()
                    .map_or(true, |s| format!("{:?}", task.status).to_lowercase() == s.to_lowercase());
                
                query_match && tags_match && status_match
            })
            .cloned()
            .collect();

        if let Some(sort_by) = &options.sort_by {
            use super::repository::{SortBy, SortOrder};
            results.sort_by(|a, b| {
                let ordering = match sort_by {
                    SortBy::Name => a.name.cmp(&b.name),
                    SortBy::CreatedAt => a.created_at.cmp(&b.created_at),
                    SortBy::SessionsCompleted => a.current_sessions.cmp(&b.current_sessions),
                    SortBy::Status => {
                        let a_ord = match a.status {
                            Status::Active => 0,
                            Status::Queued => 1,
                            Status::Paused => 2,
                            Status::Completed => 3,
                        };
                        let b_ord = match b.status {
                            Status::Active => 0,
                            Status::Queued => 1,
                            Status::Paused => 2,
                            Status::Completed => 3,
                        };
                        a_ord.cmp(&b_ord)
                    }
                };
                match options.sort_order.as_ref().unwrap_or(&SortOrder::Ascending) {
                    SortOrder::Ascending => ordering,
                    SortOrder::Descending => ordering.reverse(),
                }
            });
        }

        Ok(results)
    }

    async fn search_fuzzy(&self, query: &str) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        let query_words: Vec<String> = query.to_lowercase().split_whitespace()
            .map(|s| s.to_string()).collect();
        
        Ok(tasks
            .values()
            .filter(|task| {
                // Check if all query words match somewhere in the task
                query_words.iter().all(|word| {
                    task.name.to_lowercase().contains(word)
                        || task.description.as_ref()
                            .map_or(false, |d| d.to_lowercase().contains(word))
                        || task.tags.iter().any(|tag| tag.to_lowercase().contains(word))
                })
            })
            .cloned()
            .collect())
    }

    async fn get_incomplete_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        let mut incomplete: Vec<Task> = tasks
            .values()
            .filter(|task| !task.is_completed())
            .cloned()
            .collect();
        incomplete.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(incomplete)
    }

    async fn get_completed_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        let mut completed: Vec<Task> = tasks
            .values()
            .filter(|task| task.is_completed())
            .cloned()
            .collect();
        completed.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_return_none_when_no_default_task() {
        let repo = InMemoryRepository::new();
        let default_task = repo.get_default_task().await.unwrap();
        assert!(default_task.is_none());
    }

    #[tokio::test]
    async fn should_return_default_task() {
        let repo = InMemoryRepository::new();
        let mut task = crate::Task::new("Test Task".to_string(), 4).unwrap();
        task.set_as_default();

        repo.create(task.clone()).await.unwrap();

        let default_task = repo.get_default_task().await.unwrap();
        assert!(default_task.is_some());
        assert_eq!(default_task.unwrap().id, task.id);
    }

    #[tokio::test]
    async fn should_return_first_default_task_when_multiple_exist() {
        let repo = InMemoryRepository::new();

        // This scenario shouldn't happen in practice due to business logic,
        // but tests the repository behavior
        let mut task1 = crate::Task::new("Default 1".to_string(), 4).unwrap();
        task1.set_as_default();
        let mut task2 = crate::Task::new("Default 2".to_string(), 4).unwrap();
        task2.set_as_default();

        repo.create(task1.clone()).await.unwrap();
        repo.create(task2.clone()).await.unwrap();

        let default_task = repo.get_default_task().await.unwrap();
        assert!(default_task.is_some());
        // Should return one of them (implementation detail)
        assert!(default_task.unwrap().is_default());
    }

    #[tokio::test]
    async fn should_return_none_after_default_task_deleted() {
        let repo = InMemoryRepository::new();
        let mut task = crate::Task::new("Default Task".to_string(), 4).unwrap();
        task.set_as_default();
        let task_id = task.id;

        repo.create(task).await.unwrap();

        // Verify it exists
        let default_task = repo.get_default_task().await.unwrap();
        assert!(default_task.is_some());

        // Delete it
        repo.delete(task_id).await.unwrap();

        // Should no longer exist
        let default_task = repo.get_default_task().await.unwrap();
        assert!(default_task.is_none());
    }

    #[tokio::test]
    async fn should_find_updated_default_task() {
        let repo = InMemoryRepository::new();
        let mut task =
            crate::Task::new("Non-default Task".to_string(), 4).unwrap();

        repo.create(task.clone()).await.unwrap();

        // Initially no default
        let default_task = repo.get_default_task().await.unwrap();
        assert!(default_task.is_none());

        // Set as default and update
        task.set_as_default();
        repo.update(task.clone()).await.unwrap();

        // Should now find it
        let default_task = repo.get_default_task().await.unwrap();
        assert!(default_task.is_some());
        assert_eq!(default_task.unwrap().id, task.id);
    }
}
