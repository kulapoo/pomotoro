use domain::{Result, TimerRepository};
use std::sync::Arc;

pub async fn update_timer_secs(
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    remaining_seconds: u32,
) -> Result<()> {
    let mut timer = timer_repo.get().await?;

    timer
        .as_active_mut()
        .ok_or(domain::Error::NoActiveTask)?
        .set_remaining_seconds(remaining_seconds);

    timer_repo.save(&timer).await?;

    Ok(())
}
