use domain::{Task, TaskId, event_names};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Clone)]
pub struct TaskResource {
    pub tasks: ReadSignal<Vec<Task>>,
    pub active_task: ReadSignal<Option<Task>>,
    set_tasks: WriteSignal<Vec<Task>>,
    set_active_task: WriteSignal<Option<Task>>,
}

impl TaskResource {
    pub fn new() -> Self {
        let (tasks, set_tasks) = signal(Vec::<Task>::new());
        let (active_entity, set_active_task) = signal(None::<Task>);

        let set_tasks_clone = set_tasks;
        let set_active_task_clone = set_active_task;

        spawn_local(async move {
            web_sys::console::log_1(&"Loading all tasks...".into());
            let result =
                invoke(event_names::task::GET_ALL, JsValue::NULL).await;
            match from_value::<Vec<Task>>(result) {
                Ok(task_list) => {
                    web_sys::console::log_1(
                        &format!("Loaded {} tasks", task_list.len()).into(),
                    );
                    set_tasks_clone.set(task_list);
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to load tasks: {e}").into(),
                    );
                }
            }

            // In the new architecture, we don't have a global active task
            // Each task has its own timer
            // TODO: Load the currently active task from a different source
            set_active_task_clone.set(None);
        });

        Self {
            tasks,
            active_task: active_entity,
            set_tasks,
            set_active_task,
        }
    }

    pub async fn switch_task(&self, task_id: TaskId) -> Result<(), String> {
        // In the new architecture, switching tasks means switching which timer we're working with
        let tasks = self.tasks.get();
        let active_task = tasks.iter().find(|t| t.id == task_id).cloned();
        self.set_active_task.set(active_task);
        Ok(())
    }

    pub fn refetch(&self) {
        let _tasks = self.tasks;
        let set_tasks = self.set_tasks;
        let _set_active_task = self.set_active_task;

        spawn_local(async move {
            web_sys::console::log_1(&"Refetching tasks...".into());
            let result =
                invoke(event_names::task::GET_ALL, JsValue::NULL).await;
            match from_value::<Vec<Task>>(result) {
                Ok(task_list) => {
                    web_sys::console::log_1(
                        &format!("Refetched {} tasks", task_list.len()).into(),
                    );
                    set_tasks.set(task_list);
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to refetch tasks: {e}").into(),
                    );
                }
            }

            // In the new architecture, we don't have a global timer state
            // Each task has its own timer
            // TODO: Load the currently active task from a different source
        });
    }
}
