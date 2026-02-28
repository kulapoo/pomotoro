mod accessors;
mod commands;
mod display;

use crate::app_vm::AppViewModel;
use crate::components::ErrorInfo;
use domain::{Task, TimerState};
use leptos::prelude::*;

use crate::utils::ViewModel;

pub struct TimerViewModel {
    // These are now references to the global app state
    pub(super) timer_state: ReadSignal<TimerState>,
    pub(super) set_timer_state: WriteSignal<TimerState>,
    pub(super) active_task: ReadSignal<Option<Task>>,
    pub(super) set_active_task: WriteSignal<Option<Task>>,
    // Error state remains local to this view model
    pub(super) error_state: ReadSignal<Option<ErrorInfo>>,
    pub(super) set_error_state: WriteSignal<Option<ErrorInfo>>,
}

impl ViewModel for TimerViewModel {
    type State = TimerState;

    fn new() -> Self {
        // Get the AppViewModel from context
        let app_vm = expect_context::<StoredValue<AppViewModel>>();

        // Use the global timer and task state
        let timer_state = app_vm.with_value(|v| v.timer_state());
        let set_timer_state = app_vm.with_value(|v| v.set_timer_state);
        let active_task = app_vm.with_value(|v| v.active_task());
        let set_active_task = app_vm.with_value(|v| v.set_active_task);

        // Local error state
        let (error_state, set_error_state) = signal(None::<ErrorInfo>);

        Self {
            timer_state,
            set_timer_state,
            active_task,
            set_active_task,
            error_state,
            set_error_state,
        }
    }

    fn state(&self) -> ReadSignal<Self::State> {
        self.timer_state
    }

    fn set_state(&self) -> WriteSignal<Self::State> {
        self.set_timer_state
    }
}
