use domain::Task;
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

    pub fn get_is_idle(&self) -> bool {
        self.timer_state.get().is_idle()
    }
}
