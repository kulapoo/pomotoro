use domain::{Error, Result, TimerState, TimerStatus};

pub async fn pause_session(timer_state: &mut TimerState) -> Result<()> {
    if timer_state.status() != TimerStatus::Running {
        return Err(Error::InvalidStateTransition {
            from: format!("{:?}", timer_state.status()),
            to: "Paused".to_string(),
        });
    }

    if timer_state.active_entity_id().is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "pause_session".to_string(),
        });
    }

    // TODO: Implement pause logic using new state machine

    Ok(())
}

pub async fn resume_session(timer_state: &mut TimerState) -> Result<()> {
    if timer_state.status() != TimerStatus::Paused {
        return Err(Error::InvalidStateTransition {
            from: format!("{:?}", timer_state.status()),
            to: "Running".to_string(),
        });
    }

    if timer_state.active_entity_id().is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "resume_session".to_string(),
        });
    }

    // TODO: Implement resume logic using new state machine

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{TaskId, TimerConfiguration};

    #[tokio::test]
    async fn should_pause_running_session() {
        let task_id = TaskId::new();
        let mut timer_state = TimerState::Working {
            remaining_seconds: 1500,
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(task_id.to_string()),
            entity_session_count: 0,
        };

        pause_session(&mut timer_state).await.unwrap();

        // TODO: Fix test after implementing pause logic
    }

    #[tokio::test]
    async fn should_fail_to_pause_stopped_session() {
        let task_id = TaskId::new();
        let mut timer_state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(task_id.to_string()),
        };

        let result = pause_session(&mut timer_state).await;

        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }
}
