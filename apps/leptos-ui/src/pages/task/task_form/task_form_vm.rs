use crate::components::error_toast::{ErrorInfo, handle_command_error};
use crate::utils::{ViewModel, invoke};
use domain::{AudioConfig, Task, TaskId, event_names::commands};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::from_value;
use std::time::Duration;
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
    error_state: ReadSignal<Option<ErrorInfo>>,
    set_error_state: WriteSignal<Option<ErrorInfo>>,
}

impl ViewModel for TaskFormViewModel {
    type State = Option<Task>;

    fn new() -> Self {
        let (is_creating, set_is_creating) = signal(false);
        let (current_task, set_current_task) = signal(None::<Task>);
        let (error_state, set_error_state) = signal(None::<ErrorInfo>);

        let vm = Self {
            is_creating,
            set_is_creating,
            current_task,
            set_current_task,
            error_state,
            set_error_state,
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

                web_sys::console::log_1(
                    &"TaskCreated event received, closing form".into(),
                );
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

                from_value::<Task>(payload)
                    .ok()
                    .map(|task| set_current_task.set(Some(task)))
                    .unwrap_or_else(|| {
                        web_sys::console::error_1(
                            &"Failed to parse TaskUpdated event payload".into(),
                        );
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

    pub fn get_error_state(&self) -> Option<ErrorInfo> {
        self.error_state.get()
    }

    pub fn clear_error(&self) {
        self.set_error_state.set(None);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_task_full(
        &self,
        name: String,
        description: Option<String>,
        max_sessions: usize,
        tags: Vec<String>,
        work_duration: Option<Duration>,
        short_break_duration: Option<Duration>,
        long_break_duration: Option<Duration>,
        sessions_until_long_break: Option<u8>,
        enable_screen_blocking: Option<bool>,
        audio_config: Option<AudioConfig>,
    ) {
        let set_is_creating = self.set_is_creating;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CreateTaskRequest {
                name: String,
                description: Option<String>,
                max_sessions: u8,
                tags: Vec<String>,
                work_duration: Option<Duration>,
                short_break_duration: Option<Duration>,
                long_break_duration: Option<Duration>,
                sessions_until_long_break: Option<u8>,
                enable_screen_blocking: Option<bool>,
                audio_config: Option<AudioConfig>,
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
                work_duration,
                short_break_duration,
                long_break_duration,
                sessions_until_long_break,
                enable_screen_blocking,
                audio_config,
            };

            let args = CreateTaskArgs { request };

            if let Ok(new_task) =
                invoke::<Task, _>(commands::task::CREATE, Some(args))
                    .await
                    .map_err(|e| {
                        handle_command_error(
                            format!("Failed to create task: {}", e),
                            set_error_state,
                        );
                        set_is_creating.set(false);
                    })
            {
                web_sys::console::log_1(
                    &format!("Create task result: {:?}", new_task).into(),
                );

                web_sys::console::log_1(
                    &format!("Successfully created task: {}", new_task.name())
                        .into(),
                );
                set_is_creating.set(false);
                // Clear any existing errors on success
                set_error_state.set(None);
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_task(
        &self,
        task_id: TaskId,
        name: Option<String>,
        description: Option<String>,
        max_sessions: Option<usize>,
        tags: Option<Vec<String>>,
        work_duration: Option<Duration>,
        short_break_duration: Option<Duration>,
        long_break_duration: Option<Duration>,
        sessions_until_long_break: Option<u8>,
        enable_screen_blocking: Option<bool>,
        audio_config: Option<AudioConfig>,
    ) {
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct UpdateTaskRequest {
                id: String,
                name: Option<String>,
                description: Option<String>,
                max_sessions: Option<u8>,
                tags: Option<Vec<String>>,
                work_duration: Option<Duration>,
                short_break_duration: Option<Duration>,
                long_break_duration: Option<Duration>,
                sessions_until_long_break: Option<u8>,
                enable_screen_blocking: Option<bool>,
                audio_config: Option<AudioConfig>,
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
                work_duration,
                short_break_duration,
                long_break_duration,
                sessions_until_long_break,
                enable_screen_blocking,
                audio_config,
            };

            let args = UpdateTaskArgs { request };

            invoke::<Task, _>(commands::task::UPDATE, Some(args))
                .await
                .map(|task_dto| {
                    web_sys::console::log_1(
                        &format!("Update task result: {:?}", task_dto).into(),
                    );
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(
                        format!("Failed to update task: {}", e),
                        set_error_state,
                    );
                })
                .ok();
        });
    }
}
