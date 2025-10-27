use crate::utils::invoke;
use crate::components::error_toast::handle_command_error;
use domain::event_names::commands;
use domain::Task;
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::TaskDirectoryViewModel;

impl TaskDirectoryViewModel {
    pub fn search_tasks(&self, query: String) {
        self.set_search_query.set(query.clone());

        if query.is_empty() && self.status_filter.get() == "all" {
            self.set_filtered_tasks.set(self.tasks.get());
            return;
        }

        let set_filtered = self.set_filtered_tasks;
        let sort_by = self.sort_by.get();
        let status_filter = self.status_filter.get();
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct SearchRequest {
                query: Option<String>,
                tags: Option<Vec<String>>,
                status: Option<String>,
                sort_by: Option<String>,
                sort_order: Option<String>,
                limit: Option<usize>,
                offset: Option<usize>,
            }

            #[derive(serde::Serialize)]
            struct SearchArgs {
                request: SearchRequest,
            }

            let args = SearchArgs {
                request: SearchRequest {
                    query: if query.is_empty() { None } else { Some(query) },
                    tags: None,
                    status: if status_filter == "all" {
                        None
                    } else {
                        Some(status_filter)
                    },
                    sort_by: Some(sort_by),
                    sort_order: Some("asc".to_string()),
                    limit: None,
                    offset: None,
                },
            };

            invoke::<Vec<Task>, SearchArgs>(commands::task::SEARCH, Some(args)).await
                .map(|task_list| {
                    set_filtered.set(task_list);
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to search tasks: {}", e), set_error_state);
                })
                .ok();
        });
    }

    pub fn set_sort(&self, sort_by: String) {
        self.set_sort_by.set(sort_by);
        self.search_tasks(self.search_query.get());
    }

    pub fn set_status_filter(&self, status: String) {
        self.set_status_filter.set(status);
        self.search_tasks(self.search_query.get());
    }
}