use std::any::TypeId;

use anyhow::Result;

use domain::shared_kernel::events::AppStarted;
use usecases::bootstrap;

use crate::AppContext;

#[tokio::test]
async fn setup() -> Result<()> {
    let ctx = AppContext::with_name(Some("setup test")).await.unwrap();

    let timer_service = ctx.timer_service;
    let task_repo = ctx.task_repo;
    let event_bus = ctx.event_bus;

    bootstrap(timer_service, task_repo, event_bus.clone())
        .await
        .expect("bootstrap failure");

    assert!(ctx.db.exists());

    assert!(event_bus.has_event_type(TypeId::of::<AppStarted>()));
    Ok(())
}
