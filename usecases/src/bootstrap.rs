use std::sync::Arc;

use domain::{
    ConfigRepository, EventPublisher, Result, TaskRepository, TimerRepository,
    shared_kernel::events::AppStarted,
};

use crate::{
    task::{CreateTaskCmd, create_task},
    timer::switch_timer_task,
};

pub async fn bootstrap(
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    let task = if let Some(task) = task_repo.get_default_task().await? {
        task
    } else {
        let cmd = CreateTaskCmd {
            name: "Default Task".to_string(),
            description: None,
            max_sessions: 8,
            tags: vec![],
            config: None,
        };
        create_task(
            task_repo.clone(),
            config_repo.clone(),
            event_publisher.clone(),
            cmd,
        )
        .await?
    };

    // Check timer state and reset if not idle before switching tasks
    let timer = timer_repo.get().await?;

    if !timer.is_idle() {
        // Reset timer to idle state to allow task switching
        let mut timer = timer;
        timer.reset(&task.config.timer)?;
        timer_repo.save(&timer).await?;
    }

    switch_timer_task(
        timer_repo.clone(),
        task_repo.clone(),
        event_publisher.clone(),
        switch_timer_task::SwitchTimerTaskCmd {
            task_id: task.id.to_string(),
        },
    )
    .await?;

    let app_started = AppStarted::new(
        1,
        "v1.0.0".to_string(),
        true,
        true,
        true,
        Some(100),
        chrono::Utc::now(),
    );

    event_publisher.publish(Box::new(app_started));

    Ok(())
}
