use serde::{Deserialize, Serialize};
use crate::{TaskId, Phase, TimerStatus, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerStarted {
    pub active_task_id: Option<TaskId>,
    pub phase: Phase,
    pub duration_seconds: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TimerStarted {
    pub fn new(
        active_task_id: Option<TaskId>,
        phase: Phase,
        duration_seconds: u32,
        version: u64,
    ) -> Self {
        Self {
            active_task_id,
            phase,
            duration_seconds,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TimerStarted {
    fn event_type(&self) -> &'static str {
        "TimerStarted"
    }

    fn aggregate_id(&self) -> String {
        self.active_task_id.clone()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerPaused {
    pub active_task_id: Option<TaskId>,
    pub phase: Phase,
    pub remaining_seconds: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TimerPaused {
    pub fn new(
        active_task_id: Option<TaskId>,
        phase: Phase,
        remaining_seconds: u32,
        version: u64,
    ) -> Self {
        Self {
            active_task_id,
            phase,
            remaining_seconds,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TimerPaused {
    fn event_type(&self) -> &'static str {
        "TimerPaused"
    }

    fn aggregate_id(&self) -> String {
        self.active_task_id.clone()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerReset {
    pub active_task_id: Option<TaskId>,
    pub phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TimerReset {
    pub fn new(active_task_id: Option<TaskId>, phase: Phase, version: u64) -> Self {
        Self {
            active_task_id,
            phase,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TimerReset {
    fn event_type(&self) -> &'static str {
        "TimerReset"
    }

    fn aggregate_id(&self) -> String {
        self.active_task_id.clone()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseCompleted {
    pub active_task_id: Option<TaskId>,
    pub completed_phase: Phase,
    pub next_phase: Phase,
    pub session_count: u32,
    pub task_session_count: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl PhaseCompleted {
    pub fn new(
        active_task_id: Option<TaskId>,
        completed_phase: Phase,
        next_phase: Phase,
        session_count: u32,
        task_session_count: u32,
        version: u64,
    ) -> Self {
        Self {
            active_task_id,
            completed_phase,
            next_phase,
            session_count,
            task_session_count,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for PhaseCompleted {
    fn event_type(&self) -> &'static str {
        "PhaseCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.active_task_id.clone()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseSkipped {
    pub active_task_id: Option<TaskId>,
    pub skipped_phase: Phase,
    pub next_phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl PhaseSkipped {
    pub fn new(
        active_task_id: Option<TaskId>,
        skipped_phase: Phase,
        next_phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            active_task_id,
            skipped_phase,
            next_phase,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for PhaseSkipped {
    fn event_type(&self) -> &'static str {
        "PhaseSkipped"
    }

    fn aggregate_id(&self) -> String {
        self.active_task_id.clone()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerStatusChanged {
    pub active_task_id: Option<TaskId>,
    pub old_status: TimerStatus,
    pub new_status: TimerStatus,
    pub phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TimerStatusChanged {
    pub fn new(
        active_task_id: Option<TaskId>,
        old_status: TimerStatus,
        new_status: TimerStatus,
        phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            active_task_id,
            old_status,
            new_status,
            phase,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TimerStatusChanged {
    fn event_type(&self) -> &'static str {
        "TimerStatusChanged"
    }

    fn aggregate_id(&self) -> String {
        self.active_task_id.clone()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveTaskSwitched {
    pub old_task_id: Option<TaskId>,
    pub new_task_id: Option<TaskId>,
    pub phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl ActiveTaskSwitched {
    pub fn new(
        old_task_id: Option<TaskId>,
        new_task_id: Option<TaskId>,
        phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            old_task_id,
            new_task_id,
            phase,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for ActiveTaskSwitched {
    fn event_type(&self) -> &'static str {
        "ActiveTaskSwitched"
    }

    fn aggregate_id(&self) -> String {
        self.new_task_id.clone()
            .or(self.old_task_id.clone())
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_correct_event_types() {
        let timer_started = TimerStarted::new(Some(TaskId::new()), Phase::Work, 1500, 1);
        let timer_paused = TimerPaused::new(Some(TaskId::new()), Phase::Work, 1200, 2);
        
        assert_eq!(timer_started.event_type(), "TimerStarted");
        assert_eq!(timer_paused.event_type(), "TimerPaused");
        assert_eq!(timer_started.version(), 1);
        assert_eq!(timer_paused.version(), 2);
    }

    #[test]
    fn should_serialize_timer_started_event() {
        let event = TimerStarted::new(Some(TaskId::new()), Phase::Work, 1500, 1);

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: TimerStarted = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_phase_completed_event() {
        let event = PhaseCompleted::new(
            Some(TaskId::new()),
            Phase::Work,
            Phase::ShortBreak,
            1,
            1,
            2,
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: PhaseCompleted = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_active_task_switched_event() {
        let old_task_id = TaskId::new();
        let new_task_id = TaskId::new();
        
        let event = ActiveTaskSwitched::new(Some(old_task_id), Some(new_task_id), Phase::Work, 3);

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ActiveTaskSwitched = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}