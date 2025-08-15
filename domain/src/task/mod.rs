mod task;
pub mod builder;
pub mod task_config;
pub mod id;
pub mod status;
pub mod task_repo;
pub mod cycling_srv;
pub mod events;
#[cfg(test)]
mod test_builder;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_repository;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_cycling_service;

pub use self::task::Task;
pub use builder::TaskBuilder;
pub use task_config::TaskConfig;
pub use id::{TaskId, TaskMarker};
pub use status::TaskStatus;
pub use task_repo::TaskRepository;
pub use cycling_srv::{TaskCyclerService, TaskCyclingStrategy, DefaultTaskCyclingService};
pub use events::{
    TaskCreated, TaskSessionCompleted, TaskCompleted, TaskStatusChanged, TaskUpdated,
    SessionTransitionCompleted, TaskSwitchWorkflowCompleted, AutomaticTaskCyclingCompleted,
    TaskCyclingExhausted
};