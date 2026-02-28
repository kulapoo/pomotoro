use domain::{Timer, TimerState, event_names::commands};
use leptos::prelude::Set;
use leptos::task::spawn_local;

use crate::components::handle_command_error;
use crate::utils::invoke;

use super::AppViewModel;

impl AppViewModel {
    /// Refresh timer state from backend
    pub fn refresh_timer_state(&self) {
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            invoke::<Timer, ()>(commands::timer::GET_STATE, None)
                .await
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok()
                .map(|timer| {
                    set_timer_state.set(timer.state().clone());
                    ()
                });
        });
    }

    /// Force update timer state
    pub fn update_timer_state(&self, new_state: TimerState) {
        self.set_timer_state.set(new_state);
    }

    /// Clear any error state
    pub fn clear_error(&self) {
        self.set_error_state.set(None);
    }
}
