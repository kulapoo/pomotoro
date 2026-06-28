//! Auto-advance orchestration after a manual task complete.
//!
//! Extracted from `complete_flow.rs` so the public entry point stays a flat
//! sequence. See `complete_flow::complete_task_flow` for the full flow.

use anyhow::{Context, anyhow};
use domain::task::CycleService;
use domain::{ConfigRepository, EventPublisher, Task, TaskId, TaskRepository};
use infra::adapters::{TimerRepositoryArc, TimerTickService};
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use usecases::task::{SwitchActiveTaskCmd, switch_active_task};
use usecases::timer::{
    StartTimerPhaseCmd, reset_timer_to_idle, start_timer_phase,
};

/// Attempt to auto-advance to the next incomplete task. Returns
/// `Ok(Some(next_task_id))` on success, `Ok(None)` when AutoAdvance is off,
/// no task is eligible, or the switch failed (failures are logged), and
/// `Err(...)` when an inner step that previously propagated via `?` fails —
/// preserving the original top-level error semantics.
pub(super) async fn advance_to_next_task(
    completed_task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: TimerRepositoryArc,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    timer_tick_service: Arc<TimerTickService>,
    app_handle: &AppHandle,
) -> anyhow::Result<Option<TaskId>> {
    let Some(plan) = plan_auto_advance(&task_repo, &config_repo).await else {
        return Ok(None);
    };

    if let Err(e) = switch_active_task(
        task_repo.clone(),
        timer_repo.clone(),
        event_publisher.clone(),
        SwitchActiveTaskCmd {
            task_id: plan.next_task_id,
            old_task_id: Some(completed_task_id),
        },
    )
    .await
    .context("Failed to switch to next task after completing")
    {
        log::warn!(
            "Auto-advance after completing {} failed; staying on completed task: {e}",
            completed_task_id
        );
        return Ok(None);
    }

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
    timer_tick_service
        .stop_timer_tick_loop()
        .await
        .context("Failed to stop tick loop after auto-advance reset")?;

    let next_task = load_next_task(&task_repo, plan.next_task_id).await?;

    if plan.auto_start_work {
        maybe_start_next_work_phase(
            &plan,
            &next_task,
            task_repo.clone(),
            timer_repo.clone(),
            event_publisher.clone(),
            &timer_tick_service,
        )
        .await?;
    }

    announce_auto_advanced(
        app_handle,
        completed_task_id,
        &next_task,
        &timer_tick_service,
    )
    .await;

    Ok(Some(plan.next_task_id))
}

/// Load the next task by id, translating the `None` case into a hard error.
async fn load_next_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    id: TaskId,
) -> anyhow::Result<Task> {
    task_repo
        .get_by_id(id)
        .await
        .with_context(|| {
            format!("Failed to load task {id} after auto-advance")
        })?
        .ok_or_else(|| anyhow!("Task {id} not found after auto-advance"))
}

/// Start the work phase for the auto-advanced task.
///
/// Preserves the original asymmetry: a `start_timer_phase` failure is swallowed
/// (logged as a warning), while a `start_timer_tick_loop` failure propagates
/// and fails the whole auto-advance.
async fn maybe_start_next_work_phase(
    plan: &AdvancePlan,
    next_task: &Task,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: TimerRepositoryArc,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    timer_tick_service: &Arc<TimerTickService>,
) -> anyhow::Result<()> {
    if let Err(e) = start_timer_phase(
        task_repo,
        timer_repo,
        event_publisher,
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
        return Ok(()); // swallowed, matches original
    }

    timer_tick_service
        .start_timer_tick_loop(Some(next_task.config().timer.clone()))
        .await
        .map_err(|e| {
            anyhow!("Failed to start tick loop after auto-advance: {e}")
        })?;
    // ^ propagated as Err, matches original

    Ok(())
}

/// Emit the `AUTO_ADVANCED` event with the from/to ids, the new task, and the
/// current timer snapshot.
async fn announce_auto_advanced(
    app_handle: &AppHandle,
    from_task_id: TaskId,
    to_task: &Task,
    timer_tick_service: &Arc<TimerTickService>,
) {
    let timer_json = timer_tick_service.with_timer(|t| json!(t)).await;
    let _ = app_handle.emit(
        domain::event_names::task::AUTO_ADVANCED,
        json!({
            "from_task_id": from_task_id.to_string(),
            "to_task_id": to_task.id().to_string(),
            "to_task": to_task,
            "timer": timer_json,
        }),
    );
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
