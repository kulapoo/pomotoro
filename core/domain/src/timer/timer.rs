use super::events::*;
use super::{
    Error, Phase, Result,
    state_machine::TimerState,
    transitions::{StateTransitions, TransitionType},
};
use crate::{Event, TaskId, TimerConfiguration};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// The singleton timer row's stable primary key.
///
/// There is only ever one timer in the app; this UUID is its persistent
/// row identifier. It has nothing to do with any "default" task — the
/// name is historically unfortunate.
pub static TIMER_ROW_ID: Lazy<TaskId> = Lazy::new(|| {
    TaskId::from_string("00000000-0000-0000-0000-000000000001")
        .expect("Failed to create timer row ID")
});

/// A reference to the `Idle` state, used so that [`Timer::state`] can
/// return `&TimerState` for the task-less variant without allocation.
static IDLE_STATE: TimerState = TimerState::Idle;

/// The Pomodoro timer aggregate.
///
/// A timer is always in exactly one of two shapes, enforced by the type
/// itself:
///
/// - [`Timer::Idle`] — no task is attached. The timer cannot run; the
///   state *is* `Idle`.
/// - [`Timer::Active`] — a task is attached and the state machine may be
///   in any phase.
///
/// Because a running timer without a task is impossible by construction,
/// every state-machine command lives on [`ActiveTimer`] and never needs a
/// runtime "is there a task?" check. Callers obtain the active view via
/// [`Timer::as_active`] / [`Timer::as_active_mut`].
///
/// # Serialization
///
/// For wire/persistence compatibility, `Timer` serializes as the legacy
/// flat struct shape regardless of variant:
///
/// ```json
/// { "task_id": "<uuid or null>", "state": { "state": "Idle", ... } }
/// ```
///
/// i.e. an idle/task-less timer emits `task_id: null` and `state: Idle`.
/// This keeps the API contract stable; the `Idle`/`Active` split is an
/// internal type-level invariant, not part of the serialized form.
#[derive(Clone, Debug)]
pub enum Timer {
    Idle,
    Active(ActiveTimer),
}

/// The "has a task" shape of a [`Timer`].
///
/// Holds the bound [`TaskId`] (non-optional) and the state machine. All
/// task-requiring operations (`start`, `pause`, `tick`, …) are defined
/// here, so they cannot fail for a missing task.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActiveTimer {
    task_id: TaskId,
    state: TimerState,
}

/// Serializes as the legacy flat struct: `{ task_id, state }`.
impl Serialize for Timer {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut st = serializer.serialize_struct("Timer", 2)?;
        match self {
            Timer::Idle => {
                st.serialize_field("task_id", &None::<TaskId>)?;
                st.serialize_field("state", &IDLE_STATE)?;
            }
            Timer::Active(ActiveTimer { task_id, state }) => {
                st.serialize_field("task_id", &Some(*task_id))?;
                st.serialize_field("state", state)?;
            }
        }
        st.end()
    }
}

/// Deserializes the legacy flat struct `{ task_id, state }`. A missing
/// (`null`) `task_id` yields [`Timer::Idle`]; any in-flight state without
/// a task is discarded by the type invariant.
impl<'de> Deserialize<'de> for Timer {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct LegacyTimer {
            task_id: Option<TaskId>,
            state: TimerState,
        }

        let LegacyTimer { task_id, state } =
            LegacyTimer::deserialize(deserializer)?;
        Ok(match task_id {
            Some(id) => Timer::Active(ActiveTimer { task_id: id, state }),
            None => Timer::Idle,
        })
    }
}

impl Timer {
    /// Create a timer bound to `task_id`, starting in the `Idle` state.
    pub fn new(task_id: TaskId) -> Self {
        Timer::Active(ActiveTimer {
            task_id,
            state: TimerState::new(),
        })
    }

    /// Create a timer instance with no active task.
    ///
    /// Operations that require a task are unreachable on this variant;
    /// callers must first attach a task via [`Timer::set_task_id`] (which
    /// transitions to [`Timer::Active`]).
    pub fn idle() -> Self {
        Timer::Idle
    }

    /// Create an active timer bound to `task_id` with a pre-existing
    /// state. Used when restoring from persistence or switching the
    /// bound task while preserving state.
    pub fn with_state(task_id: TaskId, state: TimerState) -> Self {
        Timer::Active(ActiveTimer { task_id, state })
    }

    /// The bound task, if any.
    pub fn task_id(&self) -> Option<TaskId> {
        match self {
            Timer::Idle => None,
            Timer::Active(a) => Some(a.task_id),
        }
    }

    /// Attach a task to the timer.
    ///
    /// Transitions `Idle -> Active`. If a task is already attached it is
    /// replaced while preserving the current state machine.
    pub fn set_task_id(&mut self, task_id: TaskId) {
        *self = match self {
            Timer::Idle => Timer::Active(ActiveTimer {
                task_id,
                state: TimerState::new(),
            }),
            Timer::Active(ActiveTimer { state, .. }) => {
                Timer::Active(ActiveTimer {
                    task_id,
                    state: state.clone(),
                })
            }
        };
    }

    /// Detach the task from the timer and reset state to `Idle`.
    ///
    /// Used when the active task is deleted. The timer cannot run again
    /// until a new task is attached.
    pub fn clear_task_id(&mut self) {
        *self = Timer::Idle;
    }

    /// Borrow the active (task-bound) view, if any.
    pub fn as_active(&self) -> Option<&ActiveTimer> {
        match self {
            Timer::Idle => None,
            Timer::Active(a) => Some(a),
        }
    }

    /// Mutably borrow the active (task-bound) view, if any.
    ///
    /// State-machine commands are only reachable through this method,
    /// which returns `None` when no task is attached. A `None` result at
    /// a use-case boundary indicates a caller logic error and should be
    /// surfaced as `Error::NoActiveTask`.
    pub fn as_active_mut(&mut self) -> Option<&mut ActiveTimer> {
        match self {
            Timer::Idle => None,
            Timer::Active(a) => Some(a),
        }
    }

    pub fn state(&self) -> &TimerState {
        match self {
            Timer::Idle => &IDLE_STATE,
            Timer::Active(a) => &a.state,
        }
    }

    pub fn pause_from(&self) -> Option<&TimerState> {
        match self.state() {
            TimerState::Paused { paused_from, .. } => Some(paused_from),
            _ => None,
        }
    }

    pub fn can_start(&self) -> bool {
        match self {
            Timer::Idle => false,
            Timer::Active(a) => a.can_start(),
        }
    }

    pub fn can_pause(&self) -> bool {
        match self {
            Timer::Idle => false,
            Timer::Active(a) => a.can_pause(),
        }
    }

    pub fn can_resume(&self) -> bool {
        match self {
            Timer::Idle => false,
            Timer::Active(a) => a.can_resume(),
        }
    }

    pub fn can_skip(&self) -> bool {
        match self {
            Timer::Idle => false,
            Timer::Active(a) => a.can_skip(),
        }
    }

    /// Reset is allowed even with no task attached — it's a safe "go back
    /// to Idle" operation. On [`Timer::Idle`] it is a no-op; on
    /// [`Timer::Active`] it delegates to [`ActiveTimer::reset`].
    pub fn reset(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        match self {
            Timer::Idle => Ok(vec![]),
            Timer::Active(a) => a.reset(configuration),
        }
    }

    pub fn remaining_seconds(
        &self,
        configuration: Option<&TimerConfiguration>,
    ) -> u32 {
        match self.state() {
            TimerState::Idle => configuration
                .map(|c| c.get_phase_duration_seconds(Phase::Work))
                .unwrap_or(25 * 60),
            _ => self.state().remaining_seconds(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.state().is_running()
    }

    pub fn is_paused(&self) -> bool {
        self.state().is_paused()
    }

    pub fn is_idle(&self) -> bool {
        self.state().is_idle()
    }

    pub fn status(&self) -> super::Status {
        self.state().status()
    }

    pub fn get_current_phase(&self) -> Phase {
        self.state().phase()
    }
}

impl ActiveTimer {
    pub fn task_id(&self) -> TaskId {
        self.task_id
    }

    pub fn state(&self) -> &TimerState {
        &self.state
    }

    pub fn pause_from(&self) -> Option<&TimerState> {
        match &self.state {
            TimerState::Paused { paused_from, .. } => Some(paused_from),
            _ => None,
        }
    }

    pub fn remaining_seconds(
        &self,
        configuration: Option<&TimerConfiguration>,
    ) -> u32 {
        match &self.state {
            TimerState::Idle => configuration
                .map(|c| c.get_phase_duration_seconds(Phase::Work))
                .unwrap_or(25 * 60),
            _ => self.state.remaining_seconds(),
        }
    }

    pub fn set_remaining_seconds(&mut self, seconds: u32) {
        self.state = self.state.with_remaining_seconds(seconds);
    }

    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    pub fn is_paused(&self) -> bool {
        self.state.is_paused()
    }

    pub fn is_idle(&self) -> bool {
        self.state.is_idle()
    }

    pub fn status(&self) -> super::Status {
        self.state.status()
    }

    pub fn get_current_phase(&self) -> Phase {
        self.state.phase()
    }

    pub fn can_start(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Start)
    }

    pub fn can_pause(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Pause)
    }

    pub fn can_resume(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Resume)
    }

    pub fn can_skip(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Skip)
    }

    pub fn start(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Start)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Start".to_string(),
            });
        }

        let result = StateTransitions::start(
            self.state.clone(),
            self.task_id,
            configuration,
        )?;

        self.state = result.new_state;

        Ok(result.events)
    }

    pub fn pause(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Pause)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Pause".to_string(),
            });
        }

        let result = StateTransitions::pause(
            self.state.clone(),
            self.task_id,
            configuration,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn resume(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(
            &self.state,
            TransitionType::Resume,
        ) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Resume".to_string(),
            });
        }

        let result = StateTransitions::resume(
            self.state.clone(),
            self.task_id,
            configuration,
        )?;

        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn reset(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        let result = StateTransitions::reset(
            self.state.clone(),
            self.task_id,
            configuration,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn reset_phase(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        let result = StateTransitions::reset_phase(
            self.state.clone(),
            self.task_id,
            configuration,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn skip_phase(
        &mut self,
        configuration: &TimerConfiguration,
        next_phase: Phase,
    ) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Skip)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Skip".to_string(),
            });
        }

        let result = StateTransitions::skip_phase(
            self.state.clone(),
            self.task_id,
            configuration,
            next_phase,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn tick(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<(bool, Vec<Box<dyn Event>>)> {
        let (new_state, phase_complete) = StateTransitions::tick(
            self.state.clone(),
            self.task_id,
            configuration,
        )?;
        self.state = new_state.clone();

        let mut events: Vec<Box<dyn Event>> = vec![];

        let phase = self.get_current_phase();
        let tick_event = Tick::new(
            self.task_id,
            phase,
            self.state.remaining_seconds(),
            1,
            configuration.clone(),
        );

        events.push(Box::new(tick_event));

        Ok((phase_complete, events))
    }

    pub fn complete_phase(
        &mut self,
        next_phase: Phase,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(
            &self.state,
            TransitionType::CompletePhase,
        ) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "CompletePhase".to_string(),
            });
        }

        let result = StateTransitions::complete_phase(
            self.state.clone(),
            self.task_id,
            configuration,
            next_phase,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn start_phase(
        &mut self,
        phase: Phase,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        let duration = configuration.get_phase_duration_seconds(phase);
        self.state = match phase {
            Phase::Work => TimerState::Working {
                remaining_seconds: duration,
            },
            Phase::ShortBreak => TimerState::ShortBreak {
                remaining_seconds: duration,
            },
            Phase::LongBreak => TimerState::LongBreak {
                remaining_seconds: duration,
            },
        };

        let events: Vec<Box<dyn Event>> =
            vec![Box::new(Started::new(self.task_id, phase, duration, 1))];

        Ok(events)
    }

    fn get_state_name(&self) -> String {
        match &self.state {
            TimerState::Idle => "Stopped".to_string(),
            TimerState::Working { .. } => "Working".to_string(),
            TimerState::ShortBreak { .. } => "ShortBreak".to_string(),
            TimerState::LongBreak { .. } => "LongBreak".to_string(),
            TimerState::Paused { .. } => "Paused".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskId;
    use std::time::Duration;

    fn create_test_timer() -> Timer {
        Timer::new(TaskId::new())
    }

    /// Test helper: borrow the active view of a timer known to have a task.
    fn active(timer: &mut Timer) -> &mut ActiveTimer {
        timer
            .as_active_mut()
            .expect("test timer should have an active task")
    }

    fn create_test_config() -> TimerConfiguration {
        TimerConfiguration {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
        }
    }

    #[test]
    fn test_timer_creation() {
        let timer = create_test_timer();
        assert!(timer.is_idle());
        assert!(!timer.is_running());
        assert!(!timer.is_paused());
        assert!(timer.task_id().is_some());
    }

    #[test]
    fn test_idle_timer_has_no_task() {
        let timer = Timer::idle();
        assert!(timer.task_id().is_none());
        assert!(timer.is_idle());
    }

    #[test]
    fn test_idle_timer_has_no_active_view() {
        // An idle timer exposes no task-requiring operations: the type
        // makes "start without a task" unrepresentable.
        let mut timer = Timer::idle();
        assert!(timer.as_active_mut().is_none());
    }

    #[test]
    fn test_clear_task_id_resets_to_idle() {
        let mut timer = create_test_timer();
        let config = create_test_config();
        active(&mut timer).start(&config).unwrap();
        assert!(timer.is_running());

        timer.clear_task_id();
        assert!(timer.task_id().is_none());
        assert!(timer.is_idle());
    }

    #[test]
    fn test_timer_start() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        assert!(timer.can_start());
        let result = active(&mut timer).start(&config);
        assert!(result.is_ok());

        assert!(timer.is_running());
        assert!(!timer.is_idle());
        assert!(!timer.can_start());
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        active(&mut timer).start(&config).unwrap();
        assert!(timer.can_pause());

        let pause_result = active(&mut timer).pause(&config);
        assert!(pause_result.is_ok());
        assert!(timer.is_paused());
        assert!(!timer.can_pause());
        assert!(timer.can_resume());

        let resume_result = active(&mut timer).resume(&config);
        assert!(resume_result.is_ok());
        assert!(!timer.is_paused());
        assert!(timer.is_running());
    }

    #[test]
    fn test_timer_reset() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        active(&mut timer).start(&config).unwrap();
        assert!(timer.is_running());

        let reset_result = timer.reset(&config);
        assert!(reset_result.is_ok());
        assert!(timer.is_idle());
        assert!(!timer.is_running());
    }

    #[test]
    fn test_timer_reset_on_idle_is_noop() {
        let mut timer = Timer::idle();
        let config = create_test_config();
        let events = timer.reset(&config).unwrap();
        assert!(events.is_empty());
        assert!(timer.is_idle());
    }

    #[test]
    fn test_timer_skip_phase() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        active(&mut timer).start(&config).unwrap();
        assert!(timer.can_skip());

        let skip_result =
            active(&mut timer).skip_phase(&config, Phase::ShortBreak);
        assert!(skip_result.is_ok());
    }

    #[test]
    fn test_timer_remaining_seconds() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        active(&mut timer).start(&config).unwrap();
        let remaining = timer.remaining_seconds(Some(&config));
        assert!(remaining > 0);
        assert_eq!(remaining, 25 * 60);
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        active(&mut timer).start(&config).unwrap();
        let initial_remaining = timer.remaining_seconds(Some(&config));

        let tick_result = active(&mut timer).tick(&config);
        assert!(tick_result.is_ok());

        let (phase_complete, events) = tick_result.unwrap();
        assert!(!phase_complete);
        assert!(!events.is_empty());

        let new_remaining = timer.remaining_seconds(Some(&config));
        assert_eq!(new_remaining, initial_remaining - 1);
    }

    #[test]
    fn test_timer_task_id() {
        let task_id = TaskId::new();
        let timer = Timer::new(task_id);

        assert_eq!(timer.task_id(), Some(task_id));
    }

    #[test]
    fn test_set_task_id_transitions_idle_to_active() {
        let mut timer = Timer::idle();
        assert!(timer.as_active_mut().is_none());

        let id = TaskId::new();
        timer.set_task_id(id);
        assert_eq!(timer.task_id(), Some(id));
        assert!(timer.as_active_mut().is_some());
    }

    #[test]
    fn test_set_task_id_replaces_existing_task() {
        let mut timer = create_test_timer();
        let config = create_test_config();
        active(&mut timer).start(&config).unwrap();
        assert!(timer.is_running());

        let new_id = TaskId::new();
        timer.set_task_id(new_id);

        // Task replaced, running state preserved.
        assert_eq!(timer.task_id(), Some(new_id));
        assert!(timer.is_running());
    }

    #[test]
    fn test_serialize_flat_shape_active() {
        let id = TaskId::new();
        let timer = Timer::new(id);
        let json = serde_json::to_string(&timer).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["task_id"], id.as_str());
        assert_eq!(v["state"]["state"], "Idle");
        // No "Active"/"Idle" enum wrapper at the top level.
        assert!(v.get("Active").is_none());
        assert!(v.get("Idle").is_none());
    }

    #[test]
    fn test_serialize_flat_shape_idle_has_null_task() {
        let json = serde_json::to_string(&Timer::idle()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["task_id"].is_null());
        assert_eq!(v["state"]["state"], "Idle");
    }

    #[test]
    fn test_deserialize_legacy_shape_round_trips() {
        let id = TaskId::new();
        let original = Timer::with_state(
            id,
            TimerState::Working {
                remaining_seconds: 42,
            },
        );
        let json = serde_json::to_string(&original).unwrap();
        let restored: Timer = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.task_id(), Some(id));
        assert_eq!(
            restored.state(),
            &TimerState::Working {
                remaining_seconds: 42
            }
        );
    }

    #[test]
    fn test_deserialize_null_task_becomes_idle() {
        let json = r#"{"task_id":null,"state":{"state":"Working","data":{"remaining_seconds":10}}}"#;
        let timer: Timer = serde_json::from_str(json).unwrap();
        assert!(timer.task_id().is_none());
        assert!(timer.is_idle());
    }
}
