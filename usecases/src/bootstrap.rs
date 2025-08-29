use std::sync::Arc;

use domain::{
    Error, EventPublisher, Result, TaskRepository, timer::TimerService,
};

pub async fn bootstrap(
    _timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    let _default_task = if let Some(task) = task_repo.get_default_task().await? {
        task
    } else {
        let task = domain::Task::new("Default Task".to_string(), 4)
            .map_err(|e| Error::TaskCreationError {
                message: e.to_string(),
            })?
            .with_default(true);

        task_repo.create(task.clone()).await?;
        task
    };

    // timer_service
    //     .load_state()
    //     .await
    //     .context("Failed to load timer state")?;

    // // Check timer state and reset if not idle before switching tasks
    // let timer_state = timer_service
    //     .get_state()
    //     .await
    //     .context("Failed to get timer state")?;

    // if !timer_state.is_idle() {
    //     // Reset timer to idle state to allow task switching
    //     timer_service
    //         .stop_timer()
    //         .await
    //         .context("Failed to reset timer state")?;
    // }

    // switch_timer_task(
    //     &timer_service,
    //     &task_repository,
    //     &event_publisher,
    //     switch_timer_task::SwitchTimerTaskCmd {
    //         task_id: default_task.id.to_string(),
    //     },
    // )
    // .await?;

    // let app_started = app_lifecycle::AppStarted::new(
    //     1,
    //     "v1.0.0".to_string(),
    //     true,
    //     true,
    //     true,
    //     Some(100),
    //     chrono::Utc::now(),
    // );

    // event_publisher.publish(Box::new(app_started));

    Ok(())
}
