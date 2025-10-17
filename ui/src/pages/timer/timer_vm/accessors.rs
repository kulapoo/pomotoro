use domain::{StateTransitions, Task, TransitionType};
use leptos::prelude::*;

use crate::components::ErrorInfo;

use super::TimerViewModel;

// Accessors
impl TimerViewModel {
    pub fn error_state(&self) -> ReadSignal<Option<ErrorInfo>> {
        self.error_state
    }

    pub fn set_error_state(&self) -> WriteSignal<Option<ErrorInfo>> {
        self.set_error_state
    }

    pub fn get_active_task(&self) -> Option<Task> {
        self.active_task.get()
    }

    pub fn get_active_task_name(&self) -> String {
        self.active_task
            .get()
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "No Task Selected".to_string())
    }

    pub fn get_active_entity_id(&self) -> Option<String> {
        self.active_task.get().map(|task| task.id.to_string())
    }

    pub fn get_is_completed(&self) -> bool {
        self.active_task.get().map(|t| t.is_completed()).unwrap_or(false)
    }

    pub fn can_skip(&self) -> bool {
        StateTransitions::can_transition(&self.timer_state.get(), TransitionType::Skip)
    }

    pub fn can_toggle_start_pause(&self) -> bool {
        StateTransitions::can_transition(&self.timer_state.get(), TransitionType::Start)
            || StateTransitions::can_transition(&self.timer_state.get(), TransitionType::Pause)
            || StateTransitions::can_transition(&self.timer_state.get(), TransitionType::Resume)
    }

    pub fn is_task_completed(&self) -> bool {
        self.active_task.get().map(|t| t.is_completed()).unwrap_or(false)
    }

}
