mod accessors;
mod initialization;
mod commands;
mod display;

mod task_ops;

use crate::components::ErrorInfo;
use domain::{Task, TimerState};
use leptos::prelude::*;

use crate::utils::ViewModel;

pub struct TimerViewModel {
    pub(super) timer_state: ReadSignal<TimerState>,
    pub(super) set_timer_state: WriteSignal<TimerState>,
    pub(super) active_task: ReadSignal<Option<Task>>,
    pub(super) set_active_task: WriteSignal<Option<Task>>,
    pub(super) error_state: ReadSignal<Option<ErrorInfo>>,
    pub(super) set_error_state: WriteSignal<Option<ErrorInfo>>,
}

impl ViewModel for TimerViewModel {
    type State = TimerState;

    fn new() -> Self {
        let (timer_state, set_timer_state) = signal(TimerState::default());
        let (active_task, set_active_task) = signal(None::<Task>);
        let (error_state, set_error_state) = signal(None::<ErrorInfo>);

        let vm = Self {
            timer_state,
            set_timer_state,
            active_task,
            set_active_task,
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
