use domain::TaskRepository;
use std::sync::Arc;

pub type TaskRepositoryArc = Arc<dyn TaskRepository + Send + Sync>;
