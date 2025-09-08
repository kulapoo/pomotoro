use std::sync::Arc;

use domain::{
    Error, Event, Phase, Result, TaskId, TaskRepository, TimerRepository,
};

pub async fn complete_work_session(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
) -> Result<Vec<Box<dyn Event>>> {
    let mut timer = timer_repo.get().await?;
    let Some(task_id) = timer.active_task_id() else {
        return Err(Error::InvalidTaskParams {
            message: ("No active task".to_string()),
        });
    };

    let mut task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.as_str(),
        }
    })?;

    task.increment_session()?;

    let next_phase = determine_next_break_type(&task);

    let events = timer.complete_phase(next_phase, &task.config.timer)?;

    task_repo.update(task).await?;

    timer_repo.save(&timer).await?;

    Ok(events)
}

fn determine_next_break_type(task: &domain::Task) -> Phase {
    let sessions_until_long = task.config.timer.sessions_until_long_break;

    if task.current_sessions % sessions_until_long == 0 {
        Phase::LongBreak
    } else {
        Phase::ShortBreak
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Config, TaskBuilder};

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

        let next_phase = determine_next_break_type(&task);
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

        let next_phase = determine_next_break_type(&task);
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

        let next_phase = determine_next_break_type(&task);
        assert_eq!(next_phase, Phase::ShortBreak);
    }
}
