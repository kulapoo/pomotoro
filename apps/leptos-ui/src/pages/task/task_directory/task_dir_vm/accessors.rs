use crate::components::ErrorInfo;
use domain::{Task, TaskId};
use leptos::prelude::*;

use super::TaskDirectoryViewModel;

impl TaskDirectoryViewModel {
    pub fn get_tasks(&self) -> Vec<Task> {
        if self.search_query.get().is_empty()
            && self.status_filter.get() == "all"
        {
            self.tasks.get()
        } else {
            self.filtered_tasks.get()
        }
    }

    pub fn get_active_task(&self) -> Option<Task> {
        self.active_task.get()
    }

    pub fn get_selected_task(&self) -> Option<TaskId> {
        self.selected_task.get()
    }

    pub fn get_search_query(&self) -> String {
        self.search_query.get()
    }

    pub fn get_status_filter(&self) -> String {
        self.status_filter.get()
    }

    pub fn get_error_state(&self) -> Option<ErrorInfo> {
        self.error_state.get()
    }
}
