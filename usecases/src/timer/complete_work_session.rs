use domain::{Error, Event, Phase, TaskId, TimerId, TaskRepository, TimerRepository, Result};

pub struct CompleteWorkSessionRequest {
    pub task_id: TaskId,
    pub timer_id: TimerId,
}

pub async fn execute(
    task_repo: &dyn TaskRepository,
    timer_repo: &dyn TimerRepository,
    request: CompleteWorkSessionRequest,
) -> Result<Vec<Box<dyn Event>>> {
    let mut task = task_repo
        .get_by_id(request.task_id)
        .await?
        .ok_or_else(|| Error::TaskNotFound {
            id: request.task_id.to_string()
        })?;

    let mut timer = timer_repo
        .get_by_id(request.timer_id)
        .await?
        .ok_or_else(|| Error::RepositoryError {
            message: format!("Timer not found: {}", request.timer_id)
        })?;

    task.increment_session()?;
    
    let next_phase = determine_next_break_type(&task, &timer);
    
    let events = timer.complete_phase(next_phase)?;
    
    task_repo.update(task).await?;
    timer_repo.save(timer).await?;
    
    Ok(events)
}

fn determine_next_break_type(task: &domain::Task, timer: &domain::Timer) -> Phase {
    let sessions_until_long = timer.configuration().sessions_until_long_break as u8;
    
    if task.current_sessions % sessions_until_long == 0 {
        Phase::LongBreak
    } else {
        Phase::ShortBreak
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Timer, TimerConfiguration, TaskBuilder, Config};
    use std::time::Duration;

    #[test]
    fn should_determine_short_break_after_first_session() {
        let task = TaskBuilder::new()
            .id(TaskId::new())
            .timer_id(TimerId::new())
            .name("Test Task".to_string())
            .max_sessions(8)
            .current_sessions(1)
            .config(Config::default())
            .build()
            .unwrap();

        let config = TimerConfiguration {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
        };
        
        let timer = Timer::new(TimerId::new(), config);
        
        let next_phase = determine_next_break_type(&task, &timer);
        assert_eq!(next_phase, Phase::ShortBreak);
    }

    #[test]
    fn should_determine_long_break_after_fourth_session() {
        let task = TaskBuilder::new()
            .id(TaskId::new())
            .timer_id(TimerId::new())
            .name("Test Task".to_string())
            .max_sessions(8)
            .current_sessions(4)
            .config(Config::default())
            .build()
            .unwrap();

        let config = TimerConfiguration {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
        };
        
        let timer = Timer::new(TimerId::new(), config);
        
        let next_phase = determine_next_break_type(&task, &timer);
        assert_eq!(next_phase, Phase::LongBreak);
    }

    #[test]
    fn should_cycle_back_to_short_break_after_long_break() {
        let task = TaskBuilder::new()
            .id(TaskId::new())
            .timer_id(TimerId::new())
            .name("Test Task".to_string())
            .max_sessions(10)
            .current_sessions(5)
            .config(Config::default())
            .build()
            .unwrap();

        let config = TimerConfiguration {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
        };
        
        let timer = Timer::new(TimerId::new(), config);
        
        let next_phase = determine_next_break_type(&task, &timer);
        assert_eq!(next_phase, Phase::ShortBreak);
    }
}