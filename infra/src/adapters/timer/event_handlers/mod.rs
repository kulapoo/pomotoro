mod break_phase_completed;
mod countdown_expired;
mod phase_skipped;
mod registry;
mod timer_paused;
mod timer_reset;
mod timer_started;
mod timer_status_changed;
mod timer_tick;
mod work_phase_completed;

pub(super) use break_phase_completed::BreakPhaseCompletedHandler;
pub(super) use countdown_expired::CountdownExpiredHandler;
pub(super) use phase_skipped::PhaseSkippedHandler;
pub(super) use timer_paused::TimerPausedHandler;
pub(super) use timer_reset::TimerResetHandler;
pub(super) use timer_started::TimerStartedHandler;
pub(super) use timer_status_changed::TimerStatusChangedHandler;
pub(super) use timer_tick::TimerTickHandler;
pub(super) use work_phase_completed::WorkPhaseCompletedHandler;

pub use registry::register_timer_handlers;
pub use registry::unregister_timer_handlers;
