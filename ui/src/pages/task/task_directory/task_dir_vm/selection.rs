use domain::TaskId;
use leptos::prelude::*;

use super::TaskDirectoryViewModel;

impl TaskDirectoryViewModel {
    pub fn select_task(&self, task_id: Option<TaskId>) {
        self.set_selected_task.set(task_id);
    }

    pub fn clear_error(&self) {
        self.set_error_state.set(None);
    }
}