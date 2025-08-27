use domain::{Error, Result, Task, TaskCyclerService, TaskId};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CycleIncompleteTaskQuery {
    pub current_task_id: Option<String>,
    pub direction: CycleDirection,
}

#[derive(Debug, Clone)]
pub enum CycleDirection {
    Next,
    Previous,
}

#[derive(Debug, Clone)]
pub struct IncompleteCycleResult {
    pub task: Option<Task>,
    pub position: usize,
    pub total_incomplete: usize,
    pub has_more_tasks: bool,
}

pub async fn cycle_incomplete_task(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    query: CycleIncompleteTaskQuery,
) -> Result<IncompleteCycleResult> {
    let current_id = if let Some(id_str) = query.current_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| Error::TaskNotFound { id: id_str })?,
        )
    } else {
        None
    };

    let task = match query.direction {
        CycleDirection::Next => {
            cycling_service
                .cycle_to_next_incomplete_task(current_id)
                .await?
        }
        CycleDirection::Previous => {
            cycling_service
                .cycle_to_previous_incomplete_task(current_id)
                .await?
        }
    };

    let incomplete_queue = cycling_service.get_incomplete_task_queue().await?;
    let total_incomplete = incomplete_queue.len();

    let position = if let Some(ref current_task) = task {
        incomplete_queue
            .iter()
            .position(|t| t.id == current_task.id)
            .map(|p| p + 1)
            .unwrap_or(0)
    } else {
        0
    };

    let has_more_tasks = total_incomplete > 1;

    Ok(IncompleteCycleResult {
        task,
        position,
        total_incomplete,
        has_more_tasks,
    })
}

pub async fn get_incomplete_task_info(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    current_task_id: Option<String>,
) -> Result<IncompleteCycleResult> {
    let current_id = if let Some(id_str) = current_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| Error::TaskNotFound { id: id_str })?,
        )
    } else {
        None
    };

    let incomplete_queue = cycling_service.get_incomplete_task_queue().await?;
    let total_incomplete = incomplete_queue.len();

    let current_task = if let Some(id) = current_id {
        incomplete_queue.iter().find(|t| t.id == id).cloned()
    } else {
        None
    };

    let position = if let Some(ref task) = current_task {
        incomplete_queue
            .iter()
            .position(|t| t.id == task.id)
            .map(|p| p + 1)
            .unwrap_or(0)
    } else {
        0
    };

    let has_more_tasks = total_incomplete > 1;

    Ok(IncompleteCycleResult {
        task: current_task,
        position,
        total_incomplete,
        has_more_tasks,
    })
}

pub async fn get_task_cycle_position(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    task_id: String,
) -> Result<(usize, usize)> {
    let id = TaskId::from_string(&task_id)
        .map_err(|_| Error::TaskNotFound { id: task_id })?;
    
    cycling_service.get_task_cycle_position(id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{InMemoryTaskRepository, TaskRepository};

    struct TestCyclingService {
        repo: Arc<dyn TaskRepository + Send + Sync>,
        service: domain::DefaultTaskCyclingService,
    }

    #[async_trait::async_trait]
    impl TaskCyclerService for TestCyclingService {
        async fn get_next_task(
            &self,
            current_task_id: Option<TaskId>,
        ) -> Result<Option<Task>> {
            let tasks = self.repo.get_active_tasks().await?;
            Ok(self
                .service
                .find_next_task_round_robin(&tasks, current_task_id)
                .cloned())
        }

        async fn validate_task_switch(
            &self,
            task_id: TaskId,
        ) -> Result<Option<Task>> {
            let task = self.repo.get_by_id(task_id).await?;
            if let Some(ref t) = task {
                self.service.can_switch_to_task(t)?;
            }
            Ok(task)
        }

        async fn get_active_task_queue(&self) -> Result<Vec<Task>> {
            let tasks = self.repo.get_active_tasks().await?;
            Ok(self.service.filter_available_tasks(&tasks))
        }

        async fn cycle_to_next_active_task(
            &self,
            current_task_id: Option<TaskId>,
        ) -> Result<Option<Task>> {
            self.get_next_task(current_task_id).await
        }

        async fn get_previous_task(
            &self,
            current_task_id: Option<TaskId>,
        ) -> Result<Option<Task>> {
            let tasks = self.repo.get_active_tasks().await?;
            Ok(self
                .service
                .find_previous_task_round_robin(&tasks, current_task_id)
                .cloned())
        }

        async fn get_incomplete_task_queue(&self) -> Result<Vec<Task>> {
            self.repo.get_incomplete_tasks().await
        }

        async fn cycle_to_next_incomplete_task(
            &self,
            current_task_id: Option<TaskId>,
        ) -> Result<Option<Task>> {
            let tasks = self.repo.get_incomplete_tasks().await?;
            Ok(self
                .service
                .find_next_task_round_robin(&tasks, current_task_id)
                .cloned())
        }

        async fn cycle_to_previous_incomplete_task(
            &self,
            current_task_id: Option<TaskId>,
        ) -> Result<Option<Task>> {
            let tasks = self.repo.get_incomplete_tasks().await?;
            Ok(self
                .service
                .find_previous_task_round_robin(&tasks, current_task_id)
                .cloned())
        }

        async fn get_task_cycle_position(
            &self,
            task_id: TaskId,
        ) -> Result<(usize, usize)> {
            let tasks = self.repo.get_all().await?;
            Ok(self.service.find_task_cycle_position(&tasks, task_id))
        }
    }

    async fn setup() -> (Arc<dyn TaskCyclerService + Send + Sync>, Vec<Task>) {
        let repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::new());
        let cycling_service: Arc<dyn TaskCyclerService + Send + Sync> =
            Arc::new(TestCyclingService {
                repo: repo.clone(),
                service: domain::DefaultTaskCyclingService::new(),
            });

        let task1 = Task::new("Task 1".to_string(), 4).unwrap();
        let task2 = Task::new("Task 2".to_string(), 3).unwrap();
        let mut task3 = Task::new("Task 3".to_string(), 1).unwrap();
        task3.increment_session().unwrap();

        repo.create(task1.clone()).await.unwrap();
        repo.create(task2.clone()).await.unwrap();
        repo.create(task3.clone()).await.unwrap();

        (cycling_service, vec![task1, task2, task3])
    }

    #[tokio::test]
    async fn should_cycle_to_next_incomplete_task() {
        let (service, tasks) = setup().await;

        let query = CycleIncompleteTaskQuery {
            current_task_id: Some(tasks[0].id.to_string()),
            direction: CycleDirection::Next,
        };

        let result = cycle_incomplete_task(&service, query).await.unwrap();

        assert!(result.task.is_some());
        assert_eq!(result.task.unwrap().id, tasks[1].id);
        assert_eq!(result.total_incomplete, 2);
        assert_eq!(result.position, 2);
        assert!(result.has_more_tasks);
    }

    #[tokio::test]
    async fn should_cycle_to_previous_incomplete_task() {
        let (service, tasks) = setup().await;

        let query = CycleIncompleteTaskQuery {
            current_task_id: Some(tasks[1].id.to_string()),
            direction: CycleDirection::Previous,
        };

        let result = cycle_incomplete_task(&service, query).await.unwrap();

        assert!(result.task.is_some());
        assert_eq!(result.task.unwrap().id, tasks[0].id);
        assert_eq!(result.total_incomplete, 2);
        assert_eq!(result.position, 1);
        assert!(result.has_more_tasks);
    }

    #[tokio::test]
    async fn should_skip_completed_tasks() {
        let (service, tasks) = setup().await;

        let result = get_incomplete_task_info(
            &service,
            Some(tasks[0].id.to_string()),
        )
        .await
        .unwrap();

        assert_eq!(result.total_incomplete, 2);
        assert_eq!(result.position, 1);
    }
}