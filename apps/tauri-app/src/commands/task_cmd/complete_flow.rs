//! Shared orchestration for completing a task.
//!
//! Used by both the `complete_task` Tauri command and the system-tray
//! "Complete Task" menu item so the two entry points stay behaviorally
//! identical.

use anyhow::{Context, anyhow};
use domain::{ConfigRepository, EventPublisher, Task, TaskId, TaskRepository};
use infra::adapters::{TimerRepositoryArc, TimerTickService};
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use usecases::task::complete_task as complete_task_uc;
use usecases::timer::{clear_active_task, reset_timer_to_idle};

use super::auto_advance::advance_to_next_task;

/// Mark a task as completed (force-completing all sessions), stop and reset its
/// timer, and — when `AutoAdvance` is configured — switch to the next incomplete
/// task (optionally auto-starting its work phase). Returns the task that is
/// active once the flow completes.
///
/// This is the explicit, user-initiated complete path. It MUST reset the timer,
/// unlike the auto-cycling that happens inside the timer use cases (where task
/// completion during an auto-cycle must not reset the timer).
pub async fn complete_task_flow(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: TimerRepositoryArc,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    timer_tick_service: Arc<TimerTickService>,
    app_handle: AppHandle,
) -> anyhow::Result<Task> {
    reset_timer_to_idle(
        task_id,
        task_repo.clone(),
        timer_repo.clone(),
        event_publisher.clone(),
    )
    .await
    .context("Failed to reset timer to idle after completing task")?;

    // Direct STOP. No sleep, no reliance on the TimerReset event handler —
    // per the tick-loop ownership contract, orchestrators own the side effect.
    timer_tick_service
        .stop_timer_tick_loop()
        .await
        .context("Failed to stop timer tick loop while completing task")?;

    // Complete the task (all sessions).
    complete_task_uc(&task_repo, &event_publisher, task_id)
        .await
        .with_context(|| format!("Failed to complete task: {}", task_id))?;

    let active_task_id = advance_to_next_task(
        task_id,
        task_repo.clone(),
        timer_repo.clone(),
        config_repo.clone(),
        event_publisher.clone(),
        timer_tick_service.clone(),
        &app_handle,
    )
    .await?
    .unwrap_or(task_id);

    if active_task_id == task_id {
        clear_completed_active_task(task_id, timer_repo.clone(), &app_handle)
            .await;
    }

    let task = task_repo
        .get_by_id(active_task_id)
        .await
        .context("Failed to retrieve task after completing")?
        .ok_or_else(|| anyhow!("Task not found after completing"))?;

    Ok(task)
}

/// Detach the completed task from the timer so the UI can prompt for a new
/// selection. Failures are logged (not propagated) — matches the original
/// behavior where this best-effort clear never failed the whole flow.
async fn clear_completed_active_task(
    completed_task_id: TaskId,
    timer_repo: TimerRepositoryArc,
    app_handle: &AppHandle,
) {
    if let Err(e) = clear_active_task(timer_repo).await {
        log::warn!(
            "Auto-clear of completed task {} failed; timer left bound: {e}",
            completed_task_id
        );
        return;
    }
    let _ = app_handle.emit(
        domain::event_names::task::ACTIVE_TASK_CLEARED,
        json!({ "from_task_id": completed_task_id.to_string() }),
    );
}
