use crate::pages::task::types::TaskDto;
use crate::utils::{ViewModel, invoke};
use domain::{Task, TaskId, TimerConfiguration, event_names::commands};
use leptos::prelude::*;
use leptos::task::spawn_local;
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

pub struct TaskFormViewModel {
    is_creating: ReadSignal<bool>,
    set_is_creating: WriteSignal<bool>,
    current_task: ReadSignal<Option<Task>>,
    set_current_task: WriteSignal<Option<Task>>,
}

impl ViewModel for TaskFormViewModel {
    type State = Option<Task>;

    fn new() -> Self {
        let (is_creating, set_is_creating) = signal(false);
        let (current_task, set_current_task) = signal(None::<Task>);

        let vm = Self {
            is_creating,
            set_is_creating,
            current_task,
            set_current_task,
        };

        vm.setup_event_listeners();
        vm
    }

    fn state(&self) -> ReadSignal<Self::State> {
        self.current_task
    }

    fn set_state(&self) -> WriteSignal<Self::State> {
        self.set_current_task
    }
}

impl TaskFormViewModel {
    fn setup_event_listeners(&self) {
        let set_is_creating = self.set_is_creating;

        // Listen for TaskCreated event to close form after successful creation
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let _payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(&"TaskCreated event received, closing form".into());
                set_is_creating.set(false);
            });

            listen(commands::task::TASK_CREATED, &callback).await;
            callback.forget();
        });

        // Listen for TaskUpdated event to update current task
        let set_current_task = self.set_current_task;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                from_value::<TaskDto>(payload)
                    .ok()
                    .and_then(|task_dto| task_dto.to_task().ok())
                    .map(|task| set_current_task.set(Some(task)))
                    .unwrap_or_else(|| {
                        web_sys::console::error_1(&"Failed to parse TaskUpdated event payload".into());
                    });
            });

            listen(commands::task::TASK_UPDATED, &callback).await;
            callback.forget();
        });
    }

    pub fn is_creating_task(&self) -> bool {
        self.is_creating.get()
    }

    pub fn set_creating_task(&self, creating: bool) {
        self.set_is_creating.set(creating);
    }

    pub fn get_current_task(&self) -> Option<Task> {
        self.current_task.get()
    }

    pub fn set_current_task_value(&self, task: Option<Task>) {
        self.set_current_task.set(task);
    }

    pub fn create_task_full(
        &self,
        name: String,
        description: Option<String>,
        max_sessions: usize,
        tags: Vec<String>,
        custom_config: Option<TimerConfiguration>,
    ) {
        let set_is_creating = self.set_is_creating;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CreateTaskRequest {
                name: String,
                description: Option<String>,
                max_sessions: u8,
                tags: Vec<String>,
                timer_config: Option<TimerConfiguration>,
            }

            #[derive(serde::Serialize)]
            struct CreateTaskArgs {
                request: CreateTaskRequest,
            }

            let request = CreateTaskRequest {
                name: name.clone(),
                description,
                max_sessions: max_sessions as u8,
                tags,
                timer_config: custom_config,
            };

            let args = CreateTaskArgs { request };

            invoke(commands::task::CREATE, args).await
                .map(|result| {
                    web_sys::console::log_1(
                        &format!("Create task result: {:?}", result).into(),
                    );

                    from_value::<TaskDto>(result.clone())
                        .ok()
                        .and_then(|task_dto| {
                            web_sys::console::log_1(
                                &format!("Successfully deserialized TaskDto: {}", task_dto.name).into()
                            );

                            task_dto.to_task()
                                .map(|new_task| {
                                    web_sys::console::log_1(
                                        &format!("Successfully created task: {}", new_task.name).into()
                                    );
                                })
                                .map_err(|e| {
                                    web_sys::console::error_1(
                                        &format!("Failed to convert TaskDto to Task: {}", e).into()
                                    );
                                })
                                .ok()
                        })
                        .unwrap_or_else(|| {
                            web_sys::console::error_1(
                                &format!("Failed to deserialize TaskDto").into()
                            );
                        });

                    set_is_creating.set(false);
                })
                .unwrap_or_else(|e| {
                    web_sys::console::error_1(
                        &format!("Failed to invoke create_task command: {:?}", e).into()
                    );
                    set_is_creating.set(false);
                });
        });
    }

    pub fn update_task(
        &self,
        task_id: TaskId,
        name: Option<String>,
        description: Option<String>,
        max_sessions: Option<usize>,
        tags: Option<Vec<String>>,
        timer_config: Option<TimerConfiguration>,
    ) {
        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct UpdateTaskRequest {
                id: String,
                name: Option<String>,
                description: Option<String>,
                max_sessions: Option<u8>,
                tags: Option<Vec<String>>,
                timer_config: Option<TimerConfiguration>,
            }

            #[derive(serde::Serialize)]
            struct UpdateTaskArgs {
                request: UpdateTaskRequest,
            }

            let request = UpdateTaskRequest {
                id: task_id.to_string(),
                name,
                description,
                max_sessions: max_sessions.map(|s| s as u8),
                tags,
                timer_config,
            };

            let args = UpdateTaskArgs { request };

            match invoke(commands::task::UPDATE, args).await {
                Ok(result) => {
                    web_sys::console::log_1(
                        &format!("Update task result: {:?}", result).into(),
                    );
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to invoke update_task command: {:?}", e).into()
                    );
                }
            }
        });
    }
}