use std::sync::Arc;

use domain::{
    ConfigRepository, EventPublisher, Result, TaskRepository, TimerRepository,
    shared_kernel::events::AppStarted,
};
use tracing::{debug, error, info, warn};

use crate::{
    task::{CreateTaskCmd, SetDefaultTaskCmd, create_task, set_default_task},
    timer::switch_timer_task,
};

pub async fn bootstrap(
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    // Try to get or create default task
    let task = match task_repo.get_default_task().await {
        Ok(Some(task)) => {
            println!("Bootstrap: Found existing default task: {:?}", task.id);
            task
        },
        Ok(None) => {
            println!("Bootstrap: No default task found, creating one...");
            let cmd = CreateTaskCmd {
                name: "Default Task".to_string(),
                description: None,
                max_sessions: 8,
                tags: vec![],
                config: None,
            };
            let created_task = match create_task(
                task_repo.clone(),
                config_repo.clone(),
                event_publisher.clone(),
                cmd,
            )
            .await {
                Ok(task) => task,
                Err(e) => {
                    eprintln!("Bootstrap: Failed to create default task: {:?}", e);
                    return Err(e);
                }
            };

            println!("Bootstrap: Created task: {:?}", created_task.id);

            // Mark the created task as default
            let set_default_cmd = SetDefaultTaskCmd {
                task_id: created_task.id,
            };
            match set_default_task(
                &task_repo,
                &event_publisher,
                set_default_cmd,
            )
            .await {
                Ok(task) => task,
                Err(e) => {
                    eprintln!("Bootstrap: Failed to set default task: {:?}", e);
                    return Err(e);
                }
            }
        },
        Err(e) => {
            eprintln!("Bootstrap: Error getting default task: {:?}", e);
            return Err(e);
        }
    };

    println!("Bootstrap: Using task: {:?}", task.id);

    // Try to get the timer
    println!("Bootstrap: Getting timer from repository...");
    let timer = match timer_repo.get().await {
        Ok(timer) => {
            println!("Bootstrap: Successfully got timer");
            timer
        },
        Err(e) => {
            eprintln!("Bootstrap: Failed to get timer: {:?}", e);
            eprintln!("Bootstrap: Full error details: {}", e);
            return Err(e.into());
        }
    };

    println!("Bootstrap: Current timer state: idle={}", timer.is_idle());

    // Reset timer if needed
    if !timer.is_idle() {
        println!("Bootstrap: Timer is not idle, resetting...");
        let mut timer = timer;
        if let Err(e) = timer.reset(&task.config.timer) {
            eprintln!("Bootstrap: Failed to reset timer: {:?}", e);
            return Err(e.into());
        }
        timer_repo.save(&timer).await?;
        println!("Bootstrap: Timer reset successfully");
    }

    // Determine which task to use for the timer
    let task_for_timer = if !task.is_completed() {
        // Use the default task if it's not completed
        Some(task.clone())
    } else {
        // Default task is completed, try to find any incomplete task
        println!("Bootstrap: Default task is completed, looking for incomplete tasks...");
        match task_repo.get_incomplete_tasks().await {
            Ok(incomplete_tasks) if !incomplete_tasks.is_empty() => {
                println!("Bootstrap: Found {} incomplete tasks, using the first one", incomplete_tasks.len());
                Some(incomplete_tasks.into_iter().next().unwrap())
            },
            _ => {
                println!("Bootstrap: No incomplete tasks found");
                None
            }
        }
    };

    // Switch to the selected task if we found one
    if let Some(active_task) = task_for_timer {
        println!("Bootstrap: Switching timer to task {:?}", active_task.id);
        if let Err(e) = switch_timer_task(
            timer_repo.clone(),
            task_repo.clone(),
            event_publisher.clone(),
            switch_timer_task::SwitchTimerTaskCmd {
                task_id: active_task.id,
            },
        )
        .await {
            eprintln!("Bootstrap: Failed to switch timer task: {:?}", e);
            // Don't fail bootstrap if we can't switch to the task
            // The user can select a different task from the UI
            eprintln!("Bootstrap: Continuing without active task. User can select a task from UI.");
        } else {
            println!("Bootstrap: Timer task switched successfully to: {}", active_task.name);
        }
    } else {
        println!("Bootstrap: No suitable task found for timer");
        println!("Bootstrap: User can create a new task or select one from the UI");
    }

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
