use crate::{
    adapters::{DbPool, database::models::TaskDb},
    schema::tasks,
};
use async_trait::async_trait;
use diesel::prelude::*;
use domain::{
    Error, Result, Task, TaskId, TaskRepository, TaskStatus,
    task::repository::{SearchOptions, SortBy, SortOrder},
};
use std::sync::Arc;

pub struct SqliteTaskRepository {
    pool: Arc<DbPool>,
}

impl SqliteTaskRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskRepository for SqliteTaskRepository {
    async fn create(&self, task: Task) -> Result<()> {
        let task_db = TaskDb::from(task);
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        diesel::insert_into(tasks::table)
            .values(&task_db)
            .execute(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to create task: {}", e),
            })?;

        Ok(())
    }

    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let task_db = tasks::table
            .filter(tasks::id.eq(id.to_string()))
            .first::<TaskDb>(&mut conn)
            .optional()
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get task by id: {}", e),
            })?;

        match task_db {
            Some(db) => Ok(Some(Task::try_from(db)?)),
            None => Ok(None),
        }
    }

    async fn get_all(&self) -> Result<Vec<Task>> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let tasks_db = tasks::table
            .order_by(tasks::created_at.asc())
            .load::<TaskDb>(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get all tasks: {}", e),
            })?;

        tasks_db.into_iter().map(Task::try_from).collect()
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let tasks_db = tasks::table
            .filter(tasks::status.eq("active").or(tasks::status.eq("queued")))
            .order_by(tasks::created_at.asc())
            .load::<TaskDb>(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get active tasks: {}", e),
            })?;

        tasks_db.into_iter().map(Task::try_from).collect()
    }

    async fn update(&self, task: Task) -> Result<()> {
        let task_db = TaskDb::from(task.clone());
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        diesel::update(tasks::table.filter(tasks::id.eq(task.id.to_string())))
            .set(&task_db)
            .execute(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to update task: {}", e),
            })?;

        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<bool> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        // Check if task is default
        let task_db = tasks::table
            .filter(tasks::id.eq(id.to_string()))
            .first::<TaskDb>(&mut conn)
            .optional()
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to check task: {}", e),
            })?;

        if let Some(task) = task_db {
            if task.is_default {
                return Ok(false); // Don't delete default task
            }
        } else {
            return Ok(false); // Task doesn't exist
        }

        let deleted =
            diesel::delete(tasks::table.filter(tasks::id.eq(id.to_string())))
                .execute(&mut conn)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to delete task: {}", e),
                })?;

        Ok(deleted > 0)
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        // Since tags are stored as JSON string, we need to use SQL LIKE
        let mut query = tasks::table.into_boxed();
        for tag in tags {
            query = query.filter(tasks::tags.like(format!("%\"{}\"%%", tag)));
        }

        let tasks_db = query.load::<TaskDb>(&mut conn).map_err(|e| {
            Error::RepositoryError {
                message: format!("Failed to get tasks by tags: {}", e),
            }
        })?;

        tasks_db.into_iter().map(Task::try_from).collect()
    }

    async fn get_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let status_str = match status {
            TaskStatus::Active => "active",
            TaskStatus::Completed => "completed",
            TaskStatus::Paused => "paused",
            TaskStatus::Queued => "queued",
        };

        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let tasks_db = tasks::table
            .filter(tasks::status.eq(status_str))
            .load::<TaskDb>(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get tasks by status: {}", e),
            })?;

        tasks_db.into_iter().map(Task::try_from).collect()
    }

    async fn exists(&self, id: TaskId) -> Result<bool> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let exists = tasks::table
            .filter(tasks::id.eq(id.to_string()))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to check task existence: {}", e),
            })?;

        Ok(exists > 0)
    }

    async fn get_default_task(&self) -> Result<Option<Task>> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let task_db = tasks::table
            .filter(tasks::is_default.eq(true))
            .first::<TaskDb>(&mut conn)
            .optional()
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get default task: {}", e),
            })?;

        match task_db {
            Some(db) => Ok(Some(Task::try_from(db)?)),
            None => Ok(None),
        }
    }

    async fn search(&self, options: SearchOptions) -> Result<Vec<Task>> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let mut query = tasks::table.into_boxed();

        // Apply filters
        if let Some(ref search_query) = options.criteria.query {
            let pattern = format!("%{}%", search_query.to_lowercase());
            query = query.filter(
                tasks::name
                    .like(pattern.clone())
                    .or(tasks::description.like(pattern.clone()))
                    .or(tasks::tags.like(pattern)),
            );
        }

        if let Some(ref status_str) = options.criteria.status {
            query = query.filter(tasks::status.eq(status_str.to_lowercase()));
        }

        if let Some(ref filter_tags) = options.criteria.tags {
            for tag in filter_tags {
                query =
                    query.filter(tasks::tags.like(format!("%\"{}\"%%", tag)));
            }
        }

        // Apply sorting
        if let Some(sort_by) = options.sort_by {
            let order = options.sort_order.unwrap_or(SortOrder::Ascending);

            query = match (sort_by, order) {
                (SortBy::Name, SortOrder::Ascending) => {
                    query.order_by(tasks::name.asc())
                }
                (SortBy::Name, SortOrder::Descending) => {
                    query.order_by(tasks::name.desc())
                }
                (SortBy::CreatedAt, SortOrder::Ascending) => {
                    query.order_by(tasks::created_at.asc())
                }
                (SortBy::CreatedAt, SortOrder::Descending) => {
                    query.order_by(tasks::created_at.desc())
                }
                (SortBy::SessionsCompleted, SortOrder::Ascending) => {
                    query.order_by(tasks::current_sessions.asc())
                }
                (SortBy::SessionsCompleted, SortOrder::Descending) => {
                    query.order_by(tasks::current_sessions.desc())
                }
                (SortBy::Status, SortOrder::Ascending) => {
                    query.order_by(tasks::status.asc())
                }
                (SortBy::Status, SortOrder::Descending) => {
                    query.order_by(tasks::status.desc())
                }
            };
        }

        // Apply pagination
        if let Some(limit) = options.criteria.limit {
            query = query.limit(limit as i64);

            if let Some(offset) = options.criteria.offset {
                query = query.offset(offset as i64);
            }
        }

        let tasks_db = query.load::<TaskDb>(&mut conn).map_err(|e| {
            Error::RepositoryError {
                message: format!("Failed to search tasks: {}", e),
            }
        })?;

        tasks_db.into_iter().map(Task::try_from).collect()
    }

    async fn search_fuzzy(&self, query: &str) -> Result<Vec<Task>> {
        let pattern = format!("%{}%", query.to_lowercase());

        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let tasks_db = tasks::table
            .filter(
                tasks::name
                    .like(pattern.clone())
                    .or(tasks::description.like(pattern.clone()))
                    .or(tasks::tags.like(pattern)),
            )
            .order_by(tasks::created_at.desc())
            .load::<TaskDb>(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to fuzzy search tasks: {}", e),
            })?;

        tasks_db.into_iter().map(Task::try_from).collect()
    }

    async fn get_incomplete_tasks(&self) -> Result<Vec<Task>> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let tasks_db = tasks::table
            .filter(tasks::status.ne("completed"))
            .load::<TaskDb>(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get incomplete tasks: {}", e),
            })?;

        tasks_db.into_iter().map(Task::try_from).collect()
    }

    async fn get_completed_tasks(&self) -> Result<Vec<Task>> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;

        let tasks_db = tasks::table
            .filter(tasks::status.eq("completed"))
            .load::<TaskDb>(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get completed tasks: {}", e),
            })?;

        tasks_db.into_iter().map(Task::try_from).collect()
    }
}
