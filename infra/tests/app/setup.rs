use std::any::TypeId;

use anyhow::Result;

use domain::shared_kernel::events::AppStarted;
use usecases::bootstrap;

use crate::utils::setup::setup_ctx;

#[tokio::test]
async fn setup() -> Result<()> {
    let ctx = setup_ctx("setup").await;

    bootstrap(
        ctx.timer_service.clone(),
        ctx.task_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("bootstrap failure");

    assert!(ctx.db.exists());

    assert!(ctx.event_bus.has_event_type(TypeId::of::<AppStarted>()));
    Ok(())
}
