use async_trait::async_trait;
use crate::{Result, TimerState, TimerStatus, Phase, TaskId, Task};

/// Domain service abstraction for timer operations
///
/// This trait defines the core timer operations that can be performed,
/// abstracting away infrastructure concerns like persistence, notifications,
/// and background task management.
///
/// ## Clean Architecture Placement
///
/// - **Location**: Domain Layer (abstraction)
/// - **Implementation**: Infrastructure Layer (concrete implementations)
/// - **Usage**: Application Layer (orchestrates business operations)
#[async_trait]
pub trait TimerService: Send + Sync {
    /// Start the timer with optional task context
    ///
    /// This operation transitions the timer to running state and begins
    /// the countdown for the current phase.
    async fn start_timer(&self, task: Option<&Task>) -> Result<()>;

    /// Stop the timer completely
    ///
    /// This operation stops any running timer and transitions to stopped state.
    async fn stop_timer(&self) -> Result<()>;

    /// Pause or resume the timer based on current status
    ///
    /// This operation pauses a running timer or resumes a paused timer.
    async fn toggle_pause(&self) -> Result<TimerStatus>;

    /// Reset the current phase to its full duration
    ///
    /// This operation resets the current phase timer back to its full duration
    /// while preserving the current phase and task context.
    async fn reset_current_phase(&self, task: Option<&Task>) -> Result<()>;

    /// Skip to the next phase in the pomodoro cycle
    ///
    /// This operation immediately transitions to the next phase (work -> break -> work)
    /// and may trigger work session completion events.
    async fn skip_to_next_phase(&self, task: Option<&Task>) -> Result<(Phase, Phase)>;

    /// Get the current timer state
    ///
    /// This operation returns the current state of the timer including
    /// remaining time, current phase, and active task.
    async fn get_state(&self) -> Result<TimerState>;

    /// Switch the active task for the timer
    ///
    /// This operation changes the active task context and may adjust
    /// timer configuration based on task-specific settings.
    async fn switch_task(&self, task_id: TaskId, task: Option<&Task>) -> Result<()>;

    /// Load persisted timer state (for session restoration)
    ///
    /// This operation loads previously saved timer state from persistent storage.
    async fn load_state(&self) -> Result<()>;

    /// Save current timer state (for session persistence)
    ///
    /// This operation persists the current timer state to storage.
    async fn save_state(&self) -> Result<()>;
}