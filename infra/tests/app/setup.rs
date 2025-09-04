use std::any::TypeId;

use anyhow::Result;

use domain::shared_kernel::events::AppStarted;
use usecases::bootstrap;

use crate::{AppContext, core::context::setup_test_context};

#[tokio::test]
async fn setup() -> Result<()> {
    let ctx = setup_test_context("setup").await;

    let timer_service = ctx.timer_service;
    let task_repo = ctx.task_repo;
    let event_bus = ctx.event_bus;

    bootstrap(timer_service.clone(), task_repo.clone())
        .await
        .expect("bootstrap failure");

    assert!(ctx.db.exists());

    assert!(event_bus.has_event_type(TypeId::of::<AppStarted>()));
    Ok(())
}
