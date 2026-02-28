mod registry;
mod task_active_changed;
mod task_completed;
mod task_created;
mod task_deleted;
mod task_reset;
mod task_status_changed;
mod task_updated;

pub(super) use task_active_changed::TaskActiveChangedHandler;
pub(super) use task_completed::TaskCompletedHandler;
pub(super) use task_created::TaskCreatedHandler;
pub(super) use task_deleted::TaskDeletedHandler;
pub(super) use task_reset::TaskResetHandler;
pub(super) use task_status_changed::TaskStatusChangedHandler;
pub(super) use task_updated::TaskUpdatedHandler;

pub use registry::register_task_handlers;
pub use registry::unregister_task_handlers;
