use std::sync::Arc;

use domain::{
    ConfigRepository, EventPublisher, Result, TaskRepository, TimerRepository,
    shared_kernel::events::AppStarted,
};
use log::{debug, error, info};

use crate::task::{
    CreateTaskCmd, SwitchActiveTaskCmd, create_task, switch_active_task,
};

/// Application bootstrap.
///
/// - **First boot** (no tasks exist): creates a starter "Focus Session"
///   task and binds the timer to it.
/// - **Subsequent boots**: leaves the timer's task binding alone — the
///   user's last selection persists across restarts. Only resets the
///   timer if it was somehow left in a non-idle state.
pub async fn bootstrap(
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    let mut starter_task_created = false;

    // First-boot detection: no tasks exist yet.
    let existing_tasks = task_repo.get_all().await.unwrap_or_default();
    let is_first_boot = existing_tasks.is_empty();

    if is_first_boot {
        info!("First boot detected — creating starter task...");
        let cmd = CreateTaskCmd {
            name: "Focus Session".to_string(),
            description: Some(
                "Default pomodoro task for focused work".to_string(),
            ),
            max_sessions: 4,
            tags: vec!["focus".to_string()],
            config: None,
        };

        let starter_task = match create_task(
            task_repo.clone(),
            config_repo.clone(),
            event_publisher.clone(),
            cmd,
        )
        .await
        {
            Ok(task) => task,
            Err(e) => {
                error!("Failed to create starter task: {:?}", e);
                return Err(e);
            }
        };

        info!("Created starter task: {:?}", starter_task.id());
        starter_task_created = true;

        // Bind the timer to the starter task so the user can start
        // focusing immediately.
        if let Err(e) = switch_active_task(
            task_repo.clone(),
            timer_repo.clone(),
            event_publisher.clone(),
            SwitchActiveTaskCmd {
                task_id: starter_task.id(),
                old_task_id: None,
            },
        )
        .await
        {
            error!("Failed to bind timer to starter task: {:?}", e);
            // Non-fatal — the user can pick a task from the UI.
        }
    } else {
        debug!(
            "Subsequent boot with {} existing task(s) — leaving timer alone",
            existing_tasks.len()
        );
    }

    // Reset the timer if it was somehow left running. Use the global
    // config (no longer a per-task default config lookup).
    let timer = timer_repo.get().await?;
    if !timer.is_idle() {
        info!("Timer is not idle on startup, resetting...");
        let config = config_repo.get_config().await?;
        let mut timer = timer;
        if let Err(e) = timer.reset(&config.timer) {
            error!("Failed to reset timer: {:?}", e);
            return Err(e.into());
        }
        timer_repo.save(&timer).await?;
        info!("Timer reset successfully");
    }

    let app_started = AppStarted::new(
        1,
        "v1.0.0".to_string(),
        true,
        starter_task_created,
        false,
        None,
        chrono::Utc::now(),
    );

    event_publisher.publish(Box::new(app_started));

    Ok(())
}
