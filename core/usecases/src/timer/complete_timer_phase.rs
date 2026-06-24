use std::sync::Arc;

use domain::{
    Error, EventPublisher, Phase, Result, Task, TaskCompleted, TaskId,
    TaskRepository, Timer, TimerRepository,
};

pub async fn complete_timer_phase(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<(Task, Timer, Phase)> {
    let mut timer = timer_repo.get().await?;

    let mut task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.as_str(),
        }
    })?;

    let current_phase = timer.get_current_phase();

    let next_phase = match current_phase {
        Phase::Work => {
            let next = task.next_break_phase();
            if !task.is_completed() {
                task.increment_session()?;
            }
            next
        }
        Phase::ShortBreak | Phase::LongBreak => {
            // The trailing break after the last work session has been taken:
            // finalize the task (stamp completed_at) and emit TaskCompleted.
            if task.finish_break() {
                let event =
                    TaskCompleted::new(task.id(), task.max_sessions(), 1);
                event_publisher.publish(Box::new(event));
            }
            Phase::Work
        }
    };

    let events = timer
        .as_active_mut()
        .ok_or(Error::NoActiveTask)?
        .complete_phase(next_phase, &task.config().timer)?;

    task_repo.update(task.clone()).await?;
    timer_repo.save(&timer).await?;

    for event in events {
        event_publisher.publish(event);
    }

    Ok((task, timer, next_phase))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Config, TaskBuilder, TaskId};

    #[test]
    fn should_determine_short_break_after_first_session() {
        let task = TaskBuilder::new()
            .id(TaskId::new())
            .name("Test Task".to_string())
            .max_sessions(8)
            .current_sessions(1)
            .config(Config::default())
            .build()
            .unwrap();

        // Timer configuration is now managed through task.config.timer

        let next_phase = Phase::determine_next_break_type(
            task.current_sessions(),
            task.config().timer.sessions_until_long_break,
        );
        assert_eq!(next_phase, Phase::ShortBreak);
    }

    #[test]
    fn should_determine_long_break_after_fourth_session() {
        let task = TaskBuilder::new()
            .id(TaskId::new())
            .name("Test Task".to_string())
            .max_sessions(8)
            .current_sessions(4)
            .config(Config::default())
            .build()
            .unwrap();

        // Timer configuration is now managed through task.config.timer

        let next_phase = Phase::determine_next_break_type(
            task.current_sessions(),
            task.config().timer.sessions_until_long_break,
        );
        assert_eq!(next_phase, Phase::LongBreak);
    }

    #[test]
    fn should_cycle_back_to_short_break_after_long_break() {
        let task = TaskBuilder::new()
            .id(TaskId::new())
            .name("Test Task".to_string())
            .max_sessions(10)
            .current_sessions(5)
            .config(Config::default())
            .build()
            .unwrap();

        // Timer configuration is now managed through task.config.timer

        let next_phase = Phase::determine_next_break_type(
            task.current_sessions(),
            task.config().timer.sessions_until_long_break,
        );
        assert_eq!(next_phase, Phase::ShortBreak);
    }
}
