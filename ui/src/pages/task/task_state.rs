use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use domain::{Task, TaskId, TimerState, events};

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
            let result = invoke(events::task::GET_ALL, JsValue::NULL).await;
            match from_value::<Vec<Task>>(result) {
                Ok(task_list) => {
                    web_sys::console::log_1(&format!("Loaded {} tasks", task_list.len()).into());
                    set_tasks_clone.set(task_list);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load tasks: {e}").into());
                }
            }
            
            web_sys::console::log_1(&"Loading timer state...".into());
            let result = invoke(events::timer::GET_STATE, JsValue::NULL).await;
            match from_value::<TimerState>(result) {
                Ok(timer_state) => {
                    if let Some(entity_id_str) = timer_state.active_entity_id() {
                        if let Ok(task_id) = TaskId::from_string(&entity_id_str) {
                            let task_args = to_value(&task_id).unwrap();
                            let task_result = invoke(events::task::GET, task_args).await;
                            match from_value::<Task>(task_result) {
                                Ok(task) => {
                                    web_sys::console::log_1(&format!("Active task: {}", &task.name).into());
                                    set_active_task_clone.set(Some(task));
                                }
                                Err(e) => {
                                    web_sys::console::error_1(&format!("Failed to load active task: {e}").into());
                                    set_active_task_clone.set(None);
                                }
                            }
                        } else {
                            set_active_task_clone.set(None);
                        }
                    } else {
                        set_active_task_clone.set(None);
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load timer state: {e}").into());
                }
            }
        });

        Self { tasks, active_task: active_entity, set_tasks, set_active_task }
    }

    pub async fn switch_task(&self, task_id: TaskId) -> Result<(), String> {
        let args = to_value(&task_id).map_err(|e| e.to_string())?;
        let result = invoke(events::timer::SWITCH_ACTIVE_TASK, args).await;
        
        match from_value::<TimerState>(result) {
            Ok(timer_state) => {
                if let Some(entity_id_str) = timer_state.active_entity_id() {
                    if let Ok(task_id) = TaskId::from_string(&entity_id_str) {
                        let tasks = self.tasks.get();
                        let active_task = tasks.iter().find(|t| t.id == task_id).cloned();
                        self.set_active_task.set(active_task);
                    } else {
                        self.set_active_task.set(None);
                    }
                } else {
                    self.set_active_task.set(None);
                }
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn refetch(&self) {
        let tasks = self.tasks;
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
                    web_sys::console::error_1(&format!("Failed to refetch tasks: {e}").into());
                }
            }
            
            let result = invoke(events::timer::GET_STATE, JsValue::NULL).await;
            match from_value::<TimerState>(result) {
                Ok(timer_state) => {
                    web_sys::console::log_1(&"Refetched timer state".into());
                    if let Some(entity_id_str) = timer_state.active_entity_id() {
                        if let Ok(task_id) = TaskId::from_string(&entity_id_str) {
                            let tasks = tasks.get_untracked();
                            let active_task = tasks.iter().find(|t| t.id == task_id).cloned();
                            set_active_task.set(active_task);
                        } else {
                            set_active_task.set(None);
                        }
                    } else {
                        set_active_task.set(None);
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to refetch timer state: {e}").into());
                }
            }
        });
    }
}