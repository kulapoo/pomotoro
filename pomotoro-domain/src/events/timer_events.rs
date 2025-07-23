use serde::{Deserialize, Serialize};
use crate::{TaskId, Phase, TimerStatus};
use super::domain_event::DomainEventData;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerStarted {
    pub active_task_id: Option<TaskId>,
    pub phase: Phase,
    pub duration_seconds: u32,
}

impl DomainEventData for TimerStarted {
    fn event_type() -> &'static str {
        "TimerStarted"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerPaused {
    pub active_task_id: Option<TaskId>,
    pub phase: Phase,
    pub remaining_seconds: u32,
}

impl DomainEventData for TimerPaused {
    fn event_type() -> &'static str {
        "TimerPaused"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerReset {
    pub active_task_id: Option<TaskId>,
    pub phase: Phase,
}

impl DomainEventData for TimerReset {
    fn event_type() -> &'static str {
        "TimerReset"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseCompleted {
    pub active_task_id: Option<TaskId>,
    pub completed_phase: Phase,
    pub next_phase: Phase,
    pub session_count: u32,
    pub task_session_count: u32,
}

impl DomainEventData for PhaseCompleted {
    fn event_type() -> &'static str {
        "PhaseCompleted"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseSkipped {
    pub active_task_id: Option<TaskId>,
    pub skipped_phase: Phase,
    pub next_phase: Phase,
}

impl DomainEventData for PhaseSkipped {
    fn event_type() -> &'static str {
        "PhaseSkipped"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerStatusChanged {
    pub active_task_id: Option<TaskId>,
    pub old_status: TimerStatus,
    pub new_status: TimerStatus,
    pub phase: Phase,
}

impl DomainEventData for TimerStatusChanged {
    fn event_type() -> &'static str {
        "TimerStatusChanged"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveTaskSwitched {
    pub old_task_id: Option<TaskId>,
    pub new_task_id: Option<TaskId>,
    pub phase: Phase,
}

impl DomainEventData for ActiveTaskSwitched {
    fn event_type() -> &'static str {
        "ActiveTaskSwitched"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn should_have_correct_event_types() {
        assert_eq!(TimerStarted::event_type(), "TimerStarted");
        assert_eq!(TimerPaused::event_type(), "TimerPaused");
        assert_eq!(TimerReset::event_type(), "TimerReset");
        assert_eq!(PhaseCompleted::event_type(), "PhaseCompleted");
        assert_eq!(PhaseSkipped::event_type(), "PhaseSkipped");
        assert_eq!(TimerStatusChanged::event_type(), "TimerStatusChanged");
        assert_eq!(ActiveTaskSwitched::event_type(), "ActiveTaskSwitched");
    }

    #[test]
    fn should_serialize_timer_started_event() {
        let event = TimerStarted {
            active_task_id: Some(Uuid::new_v4()),
            phase: Phase::Work,
            duration_seconds: 1500,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: TimerStarted = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_phase_completed_event() {
        let event = PhaseCompleted {
            active_task_id: Some(Uuid::new_v4()),
            completed_phase: Phase::Work,
            next_phase: Phase::ShortBreak,
            session_count: 1,
            task_session_count: 1,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: PhaseCompleted = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_active_task_switched_event() {
        let old_task_id = Uuid::new_v4();
        let new_task_id = Uuid::new_v4();
        
        let event = ActiveTaskSwitched {
            old_task_id: Some(old_task_id),
            new_task_id: Some(new_task_id),
            phase: Phase::Work,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ActiveTaskSwitched = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}