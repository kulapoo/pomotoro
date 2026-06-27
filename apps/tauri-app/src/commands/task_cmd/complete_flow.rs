//! Shared orchestration for completing a task.
//!
//! Used by both the `complete_task` Tauri command and the system-tray
//! "Complete Task" menu item so the two entry points stay behaviorally
//! identical.

use anyhow::{Context, anyhow};
use domain::task::CycleService;
use domain::{ConfigRepository, EventPublisher, Task, TaskId, TaskRepository};
use infra::adapters::{TimerRepositoryArc, TimerTickService};
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use usecases::task::{
    SwitchActiveTaskCmd, complete_task as complete_task_uc, switch_active_task,
};
use usecases::timer::{
    StartTimerPhaseCmd, clear_active_task, reset_timer_to_idle,
    start_timer_phase,
};

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

    // Auto-advance: if AutoAdvance is configured, switch to the next incomplete
    // task. Whether its work phase auto-starts is governed by
    // `auto_start_work_after_break` (same flag the timer-driven cycle path uses
    // in `progress_phase`).
    let mut active_task_id = task_id;
    if let Some(plan) = plan_auto_advance(&task_repo, &config_repo).await {
        match switch_active_task(
            task_repo.clone(),
            timer_repo.clone(),
            event_publisher.clone(),
            SwitchActiveTaskCmd {
                task_id: plan.next_task_id,
                old_task_id: Some(task_id),
            },
        )
        .await
        .context("Failed to switch to next task after completing")
        {
            Ok(()) => {
                reset_timer_to_idle(
                    plan.next_task_id,
                    task_repo.clone(),
                    timer_repo.clone(),
                    event_publisher.clone(),
                )
                .await
                .context("Failed to reset timer to idle for next task")?;

                // Refresh the in-memory cache so UI payloads below are correct,
                // and ensure the loop is stopped before any start.
                timer_tick_service
                    .load_state()
                    .await
                    .context("Failed to load timer state after auto-advance")?;
                timer_tick_service.stop_timer_tick_loop().await.context(
                    "Failed to stop tick loop after auto-advance reset",
                )?;

                if plan.auto_start_work {
                    // Drive the usecase, then start the tick loop directly.
                    // TimerStartedHandler is a UI-only emitter and no longer
                    // starts the loop for us.
                    if let Err(e) = start_timer_phase(
                        task_repo.clone(),
                        timer_repo.clone(),
                        event_publisher.clone(),
                        StartTimerPhaseCmd {
                            task_id: Some(plan.next_task_id),
                        },
                    )
                    .await
                    {
                        log::warn!(
                            "Auto-start of task {} after auto-advance failed: {e}",
                            plan.next_task_id
                        );
                    } else {
                        let next_task = task_repo
                            .get_by_id(plan.next_task_id)
                            .await
                            .context(
                                "Failed to load next task for tick-loop start",
                            )?
                            .ok_or_else(|| {
                                anyhow!(
                                    "Next task {} not found after auto-advance",
                                    plan.next_task_id
                                )
                            })?;
                        timer_tick_service
                            .start_timer_tick_loop(
                                Some(next_task.config().timer.clone()),
                                None,
                            )
                            .await
                            .map_err(|e| {
                                anyhow!(
                                    "Failed to start tick loop after auto-advance: {e}"
                                )
                            })?;
                    }
                }

                active_task_id = plan.next_task_id;

                let to_task = task_repo
                    .get_by_id(plan.next_task_id)
                    .await
                    .context("Failed to load next task for auto-advanced emit")?
                    .ok_or_else(|| {
                        anyhow!(
                            "Next task {} not found after auto-advance",
                            plan.next_task_id
                        )
                    })?;

                let _ = app_handle.emit(
                    domain::event_names::task::AUTO_ADVANCED,
                    json!({
                        "from_task_id": task_id.to_string(),
                        "to_task_id": plan.next_task_id.to_string(),
                        "to_task": to_task,
                    }),
                );
            }
            Err(e) => {
                log::warn!(
                    "Auto-advance after completing {} failed; staying on completed task: {e}",
                    task_id
                );
            }
        }
    }

    // If no new task became active (Manual mode, AutoAdvance with no eligible
    // successor, or the switch failed above), detach the completed task from
    // the timer so the UI can prompt for a new selection.
    if active_task_id == task_id {
        match clear_active_task(timer_repo.clone())
            .await
            .context("Failed to clear active task after completing")
        {
            Ok(()) => {
                let _ = app_handle.emit(
                    domain::event_names::task::ACTIVE_TASK_CLEARED,
                    json!({ "from_task_id": task_id.to_string() }),
                );
            }
            Err(e) => {
                log::warn!(
                    "Auto-clear of completed task {} failed; timer left bound: {e}",
                    task_id
                );
            }
        }
    }

    let task = task_repo
        .get_by_id(active_task_id)
        .await
        .context("Failed to retrieve task after completing")?
        .ok_or_else(|| anyhow!("Task not found after completing"))?;

    Ok(task)
}

/// Decides whether to auto-advance after a manual complete.
///
/// Returns `Some(plan)` when AutoAdvance is configured and another incomplete
/// task is available; `None` otherwise (Manual mode, config read failure, or no
/// eligible task). Config/repo errors are swallowed (logged) so a transient
/// failure can never break the core complete flow.
async fn plan_auto_advance(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
) -> Option<AdvancePlan> {
    let config = match config_repo.get_config().await {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read config for auto-advance: {e}");
            return None;
        }
    };

    if !CycleService::should_auto_cycle(&config.general) {
        return None;
    }

    let active_tasks = match task_repo.get_active_tasks().await {
        Ok(tasks) => tasks,
        Err(e) => {
            log::warn!("Failed to load tasks for auto-advance: {e}");
            return None;
        }
    };

    let next = CycleService::select_next_task(
        &active_tasks,
        None,
        &config.general.task_cycling_behavior,
    )?;

    Some(AdvancePlan {
        next_task_id: next.id(),
        auto_start_work: config.general.auto_start_work_after_break,
    })
}

struct AdvancePlan {
    next_task_id: TaskId,
    auto_start_work: bool,
}
