use leptos::prelude::*;

use domain::TimerState;

// Event listeners removed as TauriEventPublisher is no longer active
// Frontend will need to poll for state updates or use a different mechanism

pub fn setup_timer_events(_set_timer_state: WriteSignal<TimerState>) {
    // No-op for now - events are not being published from backend
}

pub fn setup_phase_complete_events() {
    // No-op for now - events are not being published from backend
}