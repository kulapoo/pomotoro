mod initialization;
mod accessors;
mod task_ops;
mod search;
mod selection;

use crate::components::ErrorInfo;
use domain::{Task, TaskId};
use leptos::prelude::*;

use crate::utils::ViewModel;

pub struct TaskDirectoryViewModel {
    pub(super) tasks: ReadSignal<Vec<Task>>,
    pub(super) set_tasks: WriteSignal<Vec<Task>>,
    pub(super) filtered_tasks: ReadSignal<Vec<Task>>,
    pub(super) set_filtered_tasks: WriteSignal<Vec<Task>>,
    pub(super) active_task: ReadSignal<Option<Task>>,
    pub(super) set_active_task: WriteSignal<Option<Task>>,
    pub(super) selected_task: ReadSignal<Option<TaskId>>,
    pub(super) set_selected_task: WriteSignal<Option<TaskId>>,
    pub(super) search_query: ReadSignal<String>,
    pub(super) set_search_query: WriteSignal<String>,
    pub(super) sort_by: ReadSignal<String>,
    pub(super) set_sort_by: WriteSignal<String>,
    pub(super) status_filter: ReadSignal<String>,
    pub(super) set_status_filter: WriteSignal<String>,
    pub(super) error_state: ReadSignal<Option<ErrorInfo>>,
    pub(super) set_error_state: WriteSignal<Option<ErrorInfo>>,
}

impl ViewModel for TaskDirectoryViewModel {
    type State = Vec<Task>;

    fn new() -> Self {
        let (tasks, set_tasks) = signal(Vec::<Task>::new());
        let (filtered_tasks, set_filtered_tasks) = signal(Vec::<Task>::new());
        let (active_task, set_active_task) = signal(None::<Task>);
        let (selected_task, set_selected_task) = signal(None::<TaskId>);
        let (search_query, set_search_query) = signal(String::new());
        let (sort_by, set_sort_by) = signal("created_at".to_string());
        let (status_filter, set_status_filter) = signal("all".to_string());
        let (error_state, set_error_state) = signal(None::<ErrorInfo>);

        let vm = Self {
            tasks,
            set_tasks,
            filtered_tasks,
            set_filtered_tasks,
            active_task,
            set_active_task,
            selected_task,
            set_selected_task,
            search_query,
            set_search_query,
            sort_by,
            set_sort_by,
            status_filter,
            set_status_filter,
            error_state,
            set_error_state,
        };

        vm.load_initial_data();
        vm.setup_event_listeners();
        vm
    }

    fn state(&self) -> ReadSignal<Self::State> {
        self.tasks
    }

    fn set_state(&self) -> WriteSignal<Self::State> {
        self.set_tasks
    }
}