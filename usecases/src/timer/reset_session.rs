use domain::{Error, Result, TimerState, TimerStatus};

pub async fn reset_session(timer_state: &mut TimerState) -> Result<()> {
    // Ensure we have an active task
    if timer_state.active_entity_id().is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "reset_session".to_string(),
        });
    }

    // Cannot reset while running - must pause or stop first
    if timer_state.status() == TimerStatus::Running {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "Reset".to_string(),
        });
    }

    // TODO: Implement reset logic using new state machine

    Ok(())
}

pub async fn reset_full_session(timer_state: &mut TimerState) -> Result<()> {
    // Ensure we have an active task
    if timer_state.active_entity_id().is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "reset_full_session".to_string(),
        });
    }

    // Cannot reset while running - must pause or stop first
    if timer_state.status() == TimerStatus::Running {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "Reset".to_string(),
        });
    }

    // TODO: Implement full reset logic using new state machine

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{TaskId, TimerConfiguration};

    #[tokio::test]
    async fn should_reset_session_when_stopped() {
        let task_id = TaskId::new();
        let mut timer_state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_entity: Some(task_id.to_string()),
        };

        reset_session(&mut timer_state).await.unwrap();

        // TODO: Fix test after implementing reset logic
    }

    #[tokio::test]
    async fn should_fail_to_reset_while_running() {
        let task_id = TaskId::new();
        let mut timer_state = TimerState::Working {
            remaining_seconds: 1500,
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_entity: Some(task_id.to_string()),
            entity_session_count: 0,
        };

        let result = reset_session(&mut timer_state).await;

        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_fail_to_reset_without_active_task() {
        let mut timer_state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: None,
        };

        let result = reset_session(&mut timer_state).await;

        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }
}
