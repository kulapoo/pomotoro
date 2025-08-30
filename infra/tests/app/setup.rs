use std::sync::Arc;

use anyhow::Result;

use domain::{timer::TimerService, EventPublisher, TaskRepository};
use usecases::bootstrap;

use crate::AppContext;

#[tokio::test]
async fn setup() -> Result<()> {
    let ctx = AppContext::with_name(Some("setup test")).await.unwrap();

    let timer_service: Arc<dyn TimerService + Send + Sync> = ctx.timer_service;
    let task_repo: Arc<dyn TaskRepository + Send + Sync> = ctx.task_repo;
    let event_bus: Arc<dyn EventPublisher + Send + Sync> = ctx.event_bus;

    bootstrap(timer_service, task_repo, event_bus).await.expect("bootstrap failure");

    assert!(ctx.db.exists());

    Ok(())
}
