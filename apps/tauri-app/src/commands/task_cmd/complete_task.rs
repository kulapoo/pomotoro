use super::*;
use anyhow::{Context, anyhow};
use domain::task::CycleService;
use infra::adapters::{TimerRepositoryArc, TimerTickService};
use log::info;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use usecases::task::{
    SwitchActiveTaskCmd, complete_task as complete_task_uc, switch_active_task,
};
use usecases::timer::{
    StartTimerPhaseCmd, reset_timer_to_idle, start_timer_phase,
};

/// Mark a task as completed (force-complete all sessions).
///
/// Also stops and resets the timer for the completed task. This orchestration
/// lives in the command layer (not in the `TaskCompleted` event handler)
/// because task completion during an auto-cycle must NOT reset the timer —
/// only the explicit user-initiated complete should.
///
/// When `task_cycling_behavior` is `AutoAdvance`, the completed task is
/// immediately followed by switching to the next incomplete task (round-robin).
/// Whether the new task's work phase auto-starts is governed by
/// `auto_start_work_after_break`, mirroring the cycling branch of
/// `progress_phase`. In `Manual` mode the timer simply stays idle on the
/// completed task.
#[tauri::command(rename_all = "snake_case")]
pub async fn complete_task(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
    app_handle: AppHandle,
) -> Result<Task, String> {
    info!("Completing task: id={}", task_id);

    let task_id_parsed = TaskId::from_string(&task_id)
        .context("Invalid task ID")
        .map_err(|e| e.to_string())?;

    // Complete the task (all sessions)
    complete_task_uc(&task_repo, &event_publisher, task_id_parsed)
        .await
        .with_context(|| format!("Failed to complete task: {}", task_id))
        .map_err(|e| {
            log::error!("Failed to complete task {}: {}", task_id, e);
            e.to_string()
        })?;

    // Stop the tick loop and reset the timer to idle for this task. The
    // `TaskCompleted` event handler no longer performs this side effect
    // (it would race with auto-cycling in the timer use cases).
    timer_tick_service
        .inner()
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!("Failed to stop timer tick loop while completing task: {e}")
        })?;

    reset_timer_to_idle(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("Failed to reset timer to idle after completing task")
    .map_err(|e| e.to_string())?;

    // Drain the async Reset event handler (stop + load_state).
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Auto-advance: if AutoAdvance is configured, switch to the next
    // incomplete task. Whether its work phase auto-starts is governed by
    // `auto_start_work_after_break` (same flag the timer-driven cycle path
    // uses in `progress_phase`).
    let mut active_task_id = task_id_parsed;
    if let Some(plan) =
        plan_auto_advance(task_repo.inner(), config_repo.inner()).await
    {
        match switch_active_task(
            task_repo.inner().clone(),
            timer_repo.inner().clone(),
            event_publisher.inner().clone(),
            SwitchActiveTaskCmd {
                task_id: plan.next_task_id,
                old_task_id: Some(task_id_parsed),
            },
        )
        .await
        .context("Failed to switch to next task after completing")
        {
            Ok(()) => {
                reset_timer_to_idle(
                    plan.next_task_id,
                    task_repo.inner().clone(),
                    timer_repo.inner().clone(),
                    event_publisher.inner().clone(),
                )
                .await
                .context("Failed to reset timer to idle for next task")
                .map_err(|e| e.to_string())?;

                // Drain the async Reset event handler (stop + load_state).
                tokio::time::sleep(Duration::from_millis(100)).await;

                if plan.auto_start_work {
                    // `start_timer_phase` publishes `TimerStarted`, which the
                    // infra `TimerStartedHandler` picks up to spin the tick
                    // loop — no manual tick-loop start needed here.
                    if let Err(e) = start_timer_phase(
                        task_repo.inner().clone(),
                        timer_repo.inner().clone(),
                        event_publisher.inner().clone(),
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
                    }
                }

                active_task_id = plan.next_task_id;

                let _ = app_handle.emit(
                    domain::event_names::task::AUTO_ADVANCED,
                    json!({
                        "from_task_id": task_id,
                        "to_task_id": plan.next_task_id,
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

    let task = task_repo
        .get_by_id(active_task_id)
        .await
        .context("Failed to retrieve task after completing")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| anyhow!("Task not found after completing"))
        .map_err(|e| e.to_string())?;

    info!(
        "Successfully completed task: id={}, name={}",
        task_id,
        task.name()
    );
    Ok(task)
}

/// Decides whether to auto-advance after a manual complete.
///
/// Returns `Some(plan)` when AutoAdvance is configured and another incomplete
/// task is available; `None` otherwise (Manual mode, config read failure, or
/// no eligible task). Config/repo errors are swallowed (logged) so a transient
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
