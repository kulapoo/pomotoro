mod accessors;
mod initialization;
mod timer_events;

use crate::components::ErrorInfo;
use domain::TimerState;
use leptos::prelude::*;

use crate::utils::ViewModel;

pub struct AppViewModel {
    pub(super) timer_state: ReadSignal<TimerState>,
    pub(super) set_timer_state: WriteSignal<TimerState>,
    pub(super) error_state: ReadSignal<Option<ErrorInfo>>,
    pub(super) set_error_state: WriteSignal<Option<ErrorInfo>>,
}

impl ViewModel for AppViewModel {
    type State = TimerState;

    fn new() -> Self {
        let (timer_state, set_timer_state) = signal(TimerState::default());
        let (error_state, set_error_state) = signal(None::<ErrorInfo>);

        let vm = Self {
            timer_state,
            set_timer_state,
            error_state,
            set_error_state,
        };

        vm.initialize();
        vm
    }

    fn state(&self) -> ReadSignal<Self::State> {
        self.timer_state
    }

    fn set_state(&self) -> WriteSignal<Self::State> {
        self.set_timer_state
    }
}