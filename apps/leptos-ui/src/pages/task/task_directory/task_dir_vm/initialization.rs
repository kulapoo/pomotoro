use crate::utils::invoke;
use domain::event_names::{commands, ui_listeners::task as task_event_names};
use domain::{Task, TaskId};
use js_sys;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use super::TaskDirectoryViewModel;

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
    let tasks = invoke::<Vec<Task>, ()>(command, None)
        .await
        .ok()
        .unwrap_or_default();

    set_tasks.set(tasks);
}

impl TaskDirectoryViewModel {
    pub(super) fn setup_event_listeners(&self) {
        self.setup_task_created_listener();
        self.setup_task_updated_listener();
        self.setup_task_deleted_listener();
        self.setup_task_completed_listener();
        self.setup_active_task_changed_listener();
        self.setup_task_progress_updated_listener();
    }

    fn setup_task_created_listener(&self) {
        let set_tasks = self.set_tasks;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskCreated event received: {:?}", payload)
                        .into(),
                );

                let set_tasks_clone = set_tasks;
                spawn_local(async move {
                    refetch_all_tasks(set_tasks_clone, commands::task::GET_ALL)
                        .await;
                });
            });

            listen(commands::task::TASK_CREATED, &callback).await;
            callback.forget();
        });
    }

    fn setup_task_updated_listener(&self) {
        let set_tasks = self.set_tasks;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskUpdated event received: {:?}", payload)
                        .into(),
                );

                let set_tasks_clone = set_tasks;
                spawn_local(async move {
                    refetch_all_tasks(set_tasks_clone, commands::task::GET_ALL)
                        .await;
                });
            });

            listen(commands::task::TASK_UPDATED, &callback).await;
            callback.forget();
        });
    }

    fn setup_task_deleted_listener(&self) {
        let set_tasks = self.set_tasks;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskDeleted event received: {:?}", payload)
                        .into(),
                );

                let set_tasks_clone = set_tasks;
                spawn_local(async move {
                    refetch_all_tasks(set_tasks_clone, commands::task::GET_ALL)
                        .await;
                });
            });

            listen(commands::task::TASK_DELETED, &callback).await;
            callback.forget();
        });
    }

    fn setup_task_completed_listener(&self) {
        let set_tasks = self.set_tasks;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskCompleted event received: {:?}", payload)
                        .into(),
                );

                let set_tasks_clone = set_tasks;
                spawn_local(async move {
                    refetch_all_tasks(set_tasks_clone, commands::task::GET_ALL)
                        .await;
                });
            });

            listen(commands::task::TASK_COMPLETED, &callback).await;
            callback.forget();
        });
    }

    fn setup_active_task_changed_listener(&self) {
        let set_active_task = self.set_active_task;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!(
                        "Active task changed event received: {:?}",
                        payload
                    )
                    .into(),
                );

                if let Ok(task_id) = from_value::<TaskId>(payload) {
                    spawn_local(async move {
                        Self::fetch_task_by_id(
                            &task_id.to_string(),
                            set_active_task,
                        )
                        .await;
                    });
                }
            });

            listen(task_event_names::ACTIVE_CHANGED, &callback).await;
            callback.forget();
        });
    }

    fn setup_task_progress_updated_listener(&self) {
        let set_tasks = self.set_tasks;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!(
                        "Task progress updated event received: {:?}",
                        payload
                    )
                    .into(),
                );

                if let Ok(updated_task) = from_value::<Task>(payload) {
                    set_tasks.update(|tasks| {
                        if let Some(index) =
                            tasks.iter().position(|t| t.id == updated_task.id)
                        {
                            tasks[index] = updated_task;
                        }
                    });
                }
            });

            listen(task_event_names::PROGRESS_UPDATED, &callback).await;
            callback.forget();
        });
    }

    pub(super) fn load_initial_data(&self) {
        let set_tasks = self.set_tasks;
        let set_active_task = self.set_active_task;
        let tasks = self.tasks;

        spawn_local(async move {
            let task_list =
                invoke::<Vec<Task>, ()>(commands::task::GET_ALL, None)
                    .await
                    .ok()
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
                        .map(|task_id| {
                            let tasks_list = tasks.get_untracked();
                            let active_task = tasks_list.iter().find(|t| t.id == task_id).cloned();

                            if let Some(ref task) = active_task {
                                web_sys::console::log_1(&format!("Found active task: {}", task.name).into());
                            } else {
                                web_sys::console::log_1(&format!("Active task not found in task list for ID: {}", task_id).into());
                            }

                            active_task
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

    // Helper method to fetch task by ID
    async fn fetch_task_by_id(
        task_id: &str,
        set_active_task: WriteSignal<Option<Task>>,
    ) {
        use serde::Serialize;

        if task_id.is_empty() {
            set_active_task.set(None);
            return;
        }

        #[derive(Serialize)]
        struct GetTaskArgs {
            id: String,
        }

        let args = GetTaskArgs {
            id: task_id.to_string(),
        };

        let task = invoke::<Option<Task>, _>(commands::task::GET, Some(args))
            .await
            .ok()
            .flatten();

        set_active_task.set(task);
    }
}
