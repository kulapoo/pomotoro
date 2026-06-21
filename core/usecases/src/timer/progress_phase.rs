use std::sync::Arc;

use domain::{
    ConfigRepository, EventPublisher, Phase, Result, Task, TaskId,
    TaskRepository, Timer, TimerRepository, task::CycleService,
};

use crate::task::{SwitchActiveTaskCmd, switch_active_task};
use crate::timer::{
    StartTimerPhaseCmd, complete_timer_phase, pause_timer_phase,
    reset_timer_to_idle, start_timer_phase,
};

#[derive(Debug, Clone)]
pub struct ProgressPhaseCmd {
    pub task_id: TaskId,
    pub from_phase: Phase,
}

/// Outcome of progressing the timer after a countdown expires.
///
/// The caller (event handler) uses this to perform infrastructure-side
/// actions (start/stop the tick loop) and emit UI events.
#[derive(Debug)]
pub enum PhaseOutcome {
    /// Next phase was auto-started. Optionally cycled to a new task.
    Started {
        task: Task,
        timer: Timer,
        next_phase: Phase,
        cycled_to: Option<TaskId>,
    },
    /// Next phase was reached but paused (manual resume required).
    Paused {
        task: Task,
        timer: Timer,
        next_phase: Phase,
        cycled_to: Option<TaskId>,
    },
    /// Task completed and no more active tasks to cycle to.
    Stopped { task: Task, timer: Timer },
}

/// Core pomodoro progression: handles everything that happens after the
/// countdown reaches zero.
///
/// This orchestrator consolidates phase completion, auto-start, and task
/// auto-cycling into a single sequential call, eliminating the race condition
/// that existed when `CountdownExpiredHandler` and `BreakPhaseCompletedHandler`
/// ran concurrently and both mutated the singleton timer.
///
/// ## Decision tree
///
/// 1. Complete the current phase via `complete_timer_phase` (publishes
///    `WorkPhaseCompleted`/`BreakPhaseCompleted` for audio/notifications).
/// 2. Determine `auto_start` from config based on `from_phase`.
/// 3. If the task is completed, transitioning from a break, and auto-cycling
///    is enabled → select next task, switch, reset, and optionally start.
/// 4. Otherwise, auto-start or pause the next phase on the same task.
pub async fn progress_phase(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: ProgressPhaseCmd,
) -> Result<PhaseOutcome> {
    let config = config_repo.get_config().await?;

    let (task, timer, next_phase) = complete_timer_phase(
        cmd.task_id,
        task_repo.clone(),
        timer_repo.clone(),
        event_publisher.clone(),
    )
    .await?;

    let auto_start = match cmd.from_phase {
        Phase::Work => config.general.auto_start_breaks,
        Phase::ShortBreak | Phase::LongBreak => {
            config.general.auto_start_work_after_break
        }
    };

    let should_cycle = task.is_completed()
        && matches!(cmd.from_phase, Phase::ShortBreak | Phase::LongBreak)
        && CycleService::should_auto_cycle(&config.general);

    if should_cycle {
        let active_tasks = task_repo.get_active_tasks().await?;

        let Some(next_task) = CycleService::select_next_task(
            &active_tasks,
            Some(&cmd.task_id),
            &config.general.task_cycling_behavior,
        ) else {
            return Ok(PhaseOutcome::Stopped { task, timer });
        };

        let next_task_id = next_task.id();

        switch_active_task(
            task_repo.clone(),
            timer_repo.clone(),
            event_publisher.clone(),
            SwitchActiveTaskCmd {
                task_id: next_task_id,
                old_task_id: Some(cmd.task_id),
            },
        )
        .await?;

        reset_timer_to_idle(
            next_task_id,
            task_repo.clone(),
            timer_repo.clone(),
            event_publisher.clone(),
        )
        .await?;

        if auto_start {
            start_timer_phase(
                task_repo.clone(),
                timer_repo.clone(),
                event_publisher.clone(),
                StartTimerPhaseCmd {
                    task_id: Some(next_task_id),
                },
            )
            .await?;
        }

        let new_task = task_repo.get_by_id(next_task_id).await?.ok_or(
            domain::Error::TaskNotFound {
                id: next_task_id.as_str(),
            },
        )?;
        let new_timer = timer_repo.get().await?;

        return if auto_start {
            Ok(PhaseOutcome::Started {
                task: new_task,
                timer: new_timer,
                next_phase: Phase::Work,
                cycled_to: Some(next_task_id),
            })
        } else {
            Ok(PhaseOutcome::Paused {
                task: new_task,
                timer: new_timer,
                next_phase: Phase::Work,
                cycled_to: Some(next_task_id),
            })
        };
    }

    if auto_start {
        Ok(PhaseOutcome::Started {
            task,
            timer,
            next_phase,
            cycled_to: None,
        })
    } else {
        let timer = pause_timer_phase(
            cmd.task_id,
            task_repo.clone(),
            timer_repo.clone(),
            event_publisher.clone(),
        )
        .await?;

        Ok(PhaseOutcome::Paused {
            task,
            timer,
            next_phase,
            cycled_to: None,
        })
    }
}
