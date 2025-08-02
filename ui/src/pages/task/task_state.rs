use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use domain::{Task, TaskId, TimerStateWithTask, events};

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
        let (active_task, set_active_task) = signal(None::<Task>);

        let set_tasks_clone = set_tasks;
        let set_active_task_clone = set_active_task;
        
        spawn_local(async move {
            // Load all tasks
            web_sys::console::log_1(&"Loading all tasks...".into());
            let result = invoke(events::task::GET_ALL, JsValue::NULL).await;
            match from_value::<Vec<Task>>(result) {
                Ok(task_list) => {
                    web_sys::console::log_1(&format!("Loaded {} tasks", task_list.len()).into());
                    set_tasks_clone.set(task_list);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load tasks: {}", e).into());
                }
            }
            
            // Load timer state with active task
            web_sys::console::log_1(&"Loading timer state with task...".into());
            let result = invoke(events::timer::GET_STATE_WITH_TASK, JsValue::NULL).await;
            match from_value::<Option<Task>>(result) {
                Ok(task) => {
                    web_sys::console::log_1(&format!("Active task: {:?}", task.as_ref().map(|t| &t.name)).into());
                    set_active_task_clone.set(task);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load active task: {}", e).into());
                }
            }
        });

        Self { tasks, active_task, set_tasks, set_active_task }
    }

    pub async fn switch_task(&self, task_id: TaskId) -> Result<(), String> {
        let args = to_value(&task_id).map_err(|e| e.to_string())?;
        let result = invoke(events::timer::SWITCH_ACTIVE_TASK, args).await;
        
        match from_value::<TimerStateWithTask>(result) {
            Ok(timer_state_with_task) => {
                self.set_active_task.set(timer_state_with_task.active_task.clone());
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn refetch(&self) {
        let set_tasks = self.set_tasks;
        let set_active_task = self.set_active_task;
        
        spawn_local(async move {
            web_sys::console::log_1(&"Refetching tasks...".into());
            let result = invoke(events::task::GET_ALL, JsValue::NULL).await;
            match from_value::<Vec<Task>>(result) {
                Ok(task_list) => {
                    web_sys::console::log_1(&format!("Refetched {} tasks", task_list.len()).into());
                    set_tasks.set(task_list);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to refetch tasks: {}", e).into());
                }
            }
            
            let result = invoke(events::timer::GET_STATE_WITH_TASK, JsValue::NULL).await;
            match from_value::<TimerStateWithTask>(result) {
                Ok(timer_state_with_task) => {
                    web_sys::console::log_1(&"Refetched active task".into());
                    set_active_task.set(timer_state_with_task.active_task);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to refetch active task: {}", e).into());
                }
            }
        });
    }
}