use std::sync::Arc;

use domain::{
    shared_kernel::events::AppStarted, timer::TimerService, EventPublisher, Result, TaskRepository, TimerRepository
};

use crate::{task::{create_task, CreateTaskCmd}, timer::switch_timer_task};

pub async fn bootstrap(
    timer_service: Arc<dyn TimerService + Send + Sync>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    // let _default_task = if let Some(task) = task_repo.get_default_task().await? {
    //     task
    // } else {
    //     let task = domain::Task::new("Default Task".to_string(), 4)
    //         .map_err(|e| Error::TaskCreationError {
    //             message: e.to_string(),
    //         })?
    //         .with_default(true);

    //     task_repo.create(task.clone()).await?;
    //     task
    // };

    let task = if let Some(task) = task_repo.get_default_task().await? {
        task
    } else {
        let cmd = CreateTaskCmd {
            name: "Default Task".to_string(),
            description: None,
            max_sessions: 8,
            tags: vec![],
        };
        create_task(
            task_repo.clone(),
            timer_repo.clone(),
            event_publisher.clone(),
            cmd
        ).await?
    };

    timer_service
        .load_state()
        .await?;

    // Check timer state and reset if not idle before switching tasks
    let timer_state = timer_service
        .get_state()
        .await?;

    if !timer_state.is_idle() {
        // Reset timer to idle state to allow task switching
        timer_service
            .stop_timer()
            .await?;
    }

    switch_timer_task(
        timer_service.clone(),
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
