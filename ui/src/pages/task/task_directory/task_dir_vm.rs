use crate::pages::task::types::TaskDto;
use crate::utils::{ViewModel, invoke};
use crate::components::error_toast::{ErrorInfo, handle_command_error};
use domain::event_names::{ui_listeners::task as task_event_names, commands};
use domain::{Task, TaskId};
use js_sys;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(
        event: &str,
        callback: &Closure<dyn Fn(JsValue)>,
    ) -> JsValue;
}

// Helper function to refetch all tasks
async fn refetch_all_tasks(set_tasks: WriteSignal<Vec<Task>>, command: &str) {
    let tasks = invoke::<Vec<TaskDto>, ()>(command, None).await
        .ok()
        .map(|task_dto_list| {
            task_dto_list.into_iter()
                .filter_map(|dto| dto.to_task().ok())
                .collect()
        })
        .unwrap_or_default();

    set_tasks.set(tasks);
}

pub struct TaskDirectoryViewModel {
    tasks: ReadSignal<Vec<Task>>,
    set_tasks: WriteSignal<Vec<Task>>,
    filtered_tasks: ReadSignal<Vec<Task>>,
    set_filtered_tasks: WriteSignal<Vec<Task>>,
    active_task: ReadSignal<Option<Task>>,
    set_active_task: WriteSignal<Option<Task>>,
    selected_task: ReadSignal<Option<TaskId>>,
    set_selected_task: WriteSignal<Option<TaskId>>,
    search_query: ReadSignal<String>,
    set_search_query: WriteSignal<String>,
    sort_by: ReadSignal<String>,
    set_sort_by: WriteSignal<String>,
    status_filter: ReadSignal<String>,
    set_status_filter: WriteSignal<String>,
    error_state: ReadSignal<Option<ErrorInfo>>,
    set_error_state: WriteSignal<Option<ErrorInfo>>,
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

// ============================================================================
// Initialization
// ============================================================================

impl TaskDirectoryViewModel {
    fn setup_event_listeners(&self) {
        let set_tasks = self.set_tasks;
        let set_active_task = self.set_active_task;

        // Listen for TaskCreated event
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskCreated event received: {:?}", payload).into(),
                );

                let set_tasks_clone = set_tasks;
                spawn_local(async move {
                    refetch_all_tasks(set_tasks_clone, commands::task::GET_ALL).await;
                });
            });

            listen(commands::task::TASK_CREATED, &callback).await;
            callback.forget();
        });

        // Listen for TaskUpdated event
        let set_tasks_for_update = self.set_tasks;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskUpdated event received: {:?}", payload).into(),
                );

                let set_tasks_clone = set_tasks_for_update;
                spawn_local(async move {
                    refetch_all_tasks(set_tasks_clone, commands::task::GET_ALL).await;
                });
            });

            listen(commands::task::TASK_UPDATED, &callback).await;
            callback.forget();
        });

        // Listen for TaskDeleted event
        let set_tasks_for_delete = self.set_tasks;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskDeleted event received: {:?}", payload).into(),
                );

                let set_tasks_clone = set_tasks_for_delete;
                spawn_local(async move {
                    refetch_all_tasks(set_tasks_clone, commands::task::GET_ALL).await;
                });
            });

            listen(commands::task::TASK_DELETED, &callback).await;
            callback.forget();
        });

        // Listen for TaskCompleted event
        let set_tasks_for_complete = self.set_tasks;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskCompleted event received: {:?}", payload).into(),
                );

                let set_tasks_clone = set_tasks_for_complete;
                spawn_local(async move {
                    refetch_all_tasks(set_tasks_clone, commands::task::GET_ALL).await;
                });
            });

            listen(commands::task::TASK_COMPLETED, &callback).await;
            callback.forget();
        });

        // Listen for active task changes
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("Active task changed event received: {:?}", payload).into(),
                );

                if let Ok(task_dto) = from_value::<TaskDto>(payload.clone()) {
                    if let Ok(task) = task_dto.to_task() {
                        set_active_task.set(Some(task));
                    }
                } else if let Ok(task) = from_value::<Task>(payload) {
                    set_active_task.set(Some(task));
                }
            });

            listen(task_event_names::ACTIVE_CHANGED, &callback).await;
            callback.forget();
        });

        // Listen for task progress updates
        let set_tasks_for_progress = self.set_tasks;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("Task progress updated event received: {:?}", payload).into(),
                );

                if let Ok(task_dto) = from_value::<TaskDto>(payload.clone()) {
                    if let Ok(updated_task) = task_dto.to_task() {
                        set_tasks_for_progress.update(|tasks| {
                            if let Some(index) = tasks.iter().position(|t| t.id == updated_task.id) {
                                tasks[index] = updated_task;
                            }
                        });
                    }
                }
            });

            listen(task_event_names::PROGRESS_UPDATED, &callback).await;
            callback.forget();
        });
    }

    fn load_initial_data(&self) {
        let set_tasks = self.set_tasks;
        let set_active_task = self.set_active_task;
        let tasks = self.tasks;

        spawn_local(async move {
            let task_list = invoke::<Vec<TaskDto>, ()>(commands::task::GET_ALL, None).await
                .ok()
                .map(|task_dto_list| {
                    task_dto_list.into_iter()
                        .filter_map(|dto| dto.to_task().ok())
                        .collect()
                })
                .unwrap_or_default();

            set_tasks.set(task_list);

            // Get timer state and extract active task
            invoke::<serde_json::Value, ()>(commands::timer::GET_STATE, None).await
                .ok()
                .and_then(|timer| {
                    web_sys::console::log_1(&format!("Timer parsed: {:?}", timer).into());

                    timer.get("active_task_id")
                        .and_then(|active_task_id| {
                            web_sys::console::log_1(&format!("Active task ID from timer: {:?}", active_task_id).into());

                            if active_task_id.is_null() {
                                web_sys::console::log_1(&"Active task ID is null".into());
                                return None;
                            }

                            // Handle both string format and object format (for backwards compatibility)
                            active_task_id.as_str()
                                .map(|s| s.to_string())
                                .or_else(|| {
                                    active_task_id.as_object()
                                        .and_then(|obj| obj.get("uuid"))
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                })
                        })
                })
                .and_then(|task_id_str| {
                    domain::TaskId::from_string(&task_id_str)
                        .ok()
                        .and_then(|task_id| {
                            let tasks_list = tasks.get_untracked();
                            let active_task = tasks_list.iter().find(|t| t.id == task_id).cloned();

                            if let Some(ref task) = active_task {
                                web_sys::console::log_1(&format!("Found active task: {}", task.name).into());
                            } else {
                                web_sys::console::log_1(&format!("Active task not found in task list for ID: {}", task_id).into());
                            }

                            Some(active_task)
                        })
                        .or_else(|| {
                            web_sys::console::error_1(&"Failed to parse task ID string".into());
                            None
                        })
                })
                .map(|active_task| set_active_task.set(active_task))
                .unwrap_or_else(|| {
                    web_sys::console::log_1(&"No active_task_id field in timer info or parsing failed".into());
                    set_active_task.set(None);
                });
        });
    }
}

// ============================================================================
// State Access (Getters)
// ============================================================================

impl TaskDirectoryViewModel {
    pub fn get_tasks(&self) -> Vec<Task> {
        if self.search_query.get().is_empty() && self.status_filter.get() == "all" {
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

    pub fn get_sort_by(&self) -> String {
        self.sort_by.get()
    }

    pub fn get_status_filter(&self) -> String {
        self.status_filter.get()
    }

    pub fn get_error_state(&self) -> Option<ErrorInfo> {
        self.error_state.get()
    }
}

// ============================================================================
// Task Operations
// ============================================================================

impl TaskDirectoryViewModel {
    pub fn delete_task(&self, task_id: TaskId) -> bool {
        let task_name = self.tasks
            .get()
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "this task".to_string());

        let confirmed = leptos::prelude::window()
            .confirm_with_message(&format!("Are you sure you want to delete \"{}\"?", task_name))
            .unwrap_or(false);

        web_sys::console::log_1(&format!("Confirmed: {:?}", confirmed).into());

        if !confirmed {
            return false;
        }

        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_selected_task = self.set_selected_task;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(Serialize)]
            struct DeleteTaskArgs {
                id: String,
            }

            let args = DeleteTaskArgs {
                id: task_id.to_string(),
            };

            web_sys::console::log_1(
                &format!("Invoking delete_task for task_id: {:?}", task_id).into(),
            );

            invoke::<(), _>(commands::task::DELETE, Some(args)).await
                .map(|_result| {
                    web_sys::console::log_1(
                        &format!("Successfully deleted task: {:?}", task_id).into(),
                    );
                    let mut current_tasks = tasks.get_untracked();
                    current_tasks.retain(|t| t.id != task_id);
                    set_tasks.set(current_tasks);
                    set_selected_task.set(None);
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to delete task: {}", e), set_error_state);
                })
                .ok();
        });

        true
    }

    pub fn switch_active_task(&self, task_id: TaskId) {
        let set_active_task = self.set_active_task;
        let tasks = self.tasks;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            web_sys::console::log_1(&format!("Switching to task: {:?}", task_id).into());

            #[derive(Serialize)]
            struct SwitchTaskArgs {
                task_id: String,
            }

            let args = SwitchTaskArgs {
                task_id: task_id.to_string(),
            };

            web_sys::console::log_1(
                &format!("Invoking switch_active_task for task_id: {:?}", task_id).into(),
            );

            invoke::<serde_json::Value, _>(commands::timer::SWITCH_ACTIVE_TASK, Some(args)).await
                .map(|timer_info| {
                    web_sys::console::log_1(
                        &format!("Timer info received: {:?}", timer_info).into(),
                    );
                    let active_id = task_id;
                    let task_list = tasks.get_untracked();
                    let active_task = task_list
                        .iter()
                        .find(|t| t.id == active_id)
                        .cloned();
                    let task_name = active_task.as_ref().map(|t| t.name.clone());
                    set_active_task.set(active_task);
                    web_sys::console::log_1(
                        &format!("Active task set to: {:?}", task_name).into(),
                    );
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to switch active task: {}", e), set_error_state);
                })
                .ok();
        });
    }

    pub fn cycle_to_next_incomplete_task(&self) {
        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }

            let current_id = tasks.get_untracked().first().map(|t| t.id.to_string());

            let args = CycleArgs {
                current_task_id: current_id,
                direction: "next".to_string(),
            };

            #[derive(serde::Deserialize)]
            struct CycleResult {
                task: Option<Task>,
                position: usize,
                total_incomplete: usize,
            }

            invoke::<CycleResult, _>(commands::task::CYCLE_INCOMPLETE_TASK, Some(args)).await
                .map(|_cycle_result| {
                    spawn_local(async move {
                        refetch_all_tasks(set_tasks, commands::task::GET_ALL).await;
                    });
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to cycle to next incomplete task: {}", e), set_error_state);
                })
                .ok();
        });
    }

    pub fn cycle_to_previous_incomplete_task(&self) {
        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }

            let current_id = tasks.get_untracked().first().map(|t| t.id.to_string());

            let args = CycleArgs {
                current_task_id: current_id,
                direction: "previous".to_string(),
            };

            #[derive(serde::Deserialize)]
            struct CycleResult {
                task: Option<Task>,
                position: usize,
                total_incomplete: usize,
            }

            invoke::<CycleResult, _>(commands::task::CYCLE_INCOMPLETE_TASK, Some(args)).await
                .map(|_cycle_result| {
                    spawn_local(async move {
                        refetch_all_tasks(set_tasks, commands::task::GET_ALL).await;
                    });
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to cycle to previous incomplete task: {}", e), set_error_state);
                })
                .ok();
        });
    }

    pub fn reset_task_to_queued(&self, task_id: TaskId) {
        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct ResetTaskStatusArgs {
                task_id: String,
            }

            let args = ResetTaskStatusArgs {
                task_id: task_id.to_string(),
            };

            invoke::<TaskDto, _>(commands::task::RESET_TASK, Some(args)).await
                .map_err(|e| {
                    handle_command_error(format!("Failed to reset task status: {}", e), set_error_state);
                })
                .ok()
                .and_then(|task_dto| {
                    web_sys::console::log_1(
                        &format!("Reset task status result: {:?}", task_dto).into(),
                    );

                    task_dto.to_task()
                        .map_err(|e| {
                            handle_command_error(format!("Failed to convert TaskDto to Task: {}", e), set_error_state);
                            spawn_local(async move {
                                refetch_all_tasks(set_tasks, commands::task::GET_ALL).await;
                            });
                        })
                        .ok()
                        .map(|updated_task| {
                            web_sys::console::log_1(
                                &format!(
                                    "Successfully reset task: id={}, status={:?}",
                                    updated_task.id, updated_task.status
                                )
                                .into(),
                            );
                            let mut current_tasks = tasks.get_untracked();
                            if let Some(index) =
                                current_tasks.iter().position(|t| t.id == task_id)
                            {
                                current_tasks[index] = updated_task;
                                set_tasks.set(current_tasks);
                            }
                            // Clear any existing errors on success
                            set_error_state.set(None);
                        })
                });
        });
    }
}

// ============================================================================
// Search & Filter Operations
// ============================================================================

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

// ============================================================================
// Selection & Error Management
// ============================================================================

impl TaskDirectoryViewModel {
    pub fn select_task(&self, task_id: Option<TaskId>) {
        self.set_selected_task.set(task_id);
    }

    pub fn clear_error(&self) {
        self.set_error_state.set(None);
    }
}