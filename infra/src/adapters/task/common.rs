use std::sync::Arc;
use domain::TaskRepository;

pub type TaskRepositoryArc = Arc<dyn TaskRepository + Send + Sync>;