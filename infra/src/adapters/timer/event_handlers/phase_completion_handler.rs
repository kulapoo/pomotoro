use tauri::{AppHandle, Emitter};
use domain::Phase;
use domain::timer::events::PhaseCompleted;
use domain::events::timer as TimerEvents;
use crate::adapters::notifications::send_phase_notification;

/// Handler for phase completion events
///
/// This handler is responsible for side effects that occur when a timer phase completes:
/// - System notifications to the user
/// - UI event emission for frontend updates
///
/// By extracting this from TimerService, we achieve better separation of concerns
/// and make the timer logic more focused and testable.
#[derive(Clone)]
pub struct PhaseCompletionHandler {
    pub app_handle: AppHandle,
}

impl PhaseCompletionHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Handle a phase completion with all related side effects
    pub fn handle_phase_completion(
        &self,
        old_phase: &Phase,
        new_phase: &Phase,
        event: &PhaseCompleted,
    ) {
        // Send system notification
        send_phase_notification(&self.app_handle, old_phase, new_phase);

        // Emit timer events for UI
        let _ = self.app_handle.emit(
            TimerEvents::PHASE_COMPLETE,
            (old_phase, new_phase),
        );

        // Emit structured event with full context
        let _ = self.app_handle.emit(TimerEvents::PHASE_EVENT, event);
    }

    /// Handle timer state updates (for UI reactivity)
    pub fn handle_timer_tick(&self, remaining_seconds: u32) {
        let _ = self.app_handle.emit(TimerEvents::TICK, remaining_seconds);
    }

    /// Handle timer status changes
    pub fn handle_status_change(&self, new_status: domain::TimerStatus) {
        let _ = self.app_handle.emit(TimerEvents::STATUS_CHANGED, new_status);
    }
}