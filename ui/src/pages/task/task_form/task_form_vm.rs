use crate::pages::task::types::TaskDto;
use crate::utils::{ViewModel, invoke_command};
use domain::{Task, TaskId, TimerConfiguration, event_names};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::{from_value, to_value};
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
    cycle_position: ReadSignal<(usize, usize)>,
    set_cycle_position: WriteSignal<(usize, usize)>,
}

impl ViewModel for TaskFormViewModel {
    type State = Option<Task>;

    fn new() -> Self {
        let (is_creating, set_is_creating) = signal(false);
        let (current_task, set_current_task) = signal(None::<Task>);
        let (cycle_position, set_cycle_position) = signal((0, 0));

        let vm = Self {
            is_creating,
            set_is_creating,
            current_task,
            set_current_task,
            cycle_position,
            set_cycle_position,
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

            listen(event_names::task::TASK_CREATED, &callback).await;
            callback.forget();
        });

        // Listen for TaskUpdated event to update current task
        let set_current_task = self.set_current_task;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                if let Ok(task_dto) = from_value::<TaskDto>(payload) {
                    if let Ok(task) = task_dto.to_task() {
                        set_current_task.set(Some(task));
                    }
                }
            });

            listen(event_names::task::TASK_UPDATED, &callback).await;
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

            match to_value(&args) {
                Ok(args_value) => {
                    web_sys::console::log_1(
                        &format!("Invoking create_task with args: {:?}", args_value).into(),
                    );
                    match invoke_command(event_names::task::CREATE, args_value).await {
                        Ok(result) => {
                            web_sys::console::log_1(
                                &format!("Create task result: {:?}", result).into(),
                            );
                            match from_value::<TaskDto>(result.clone()) {
                                Ok(task_dto) => {
                                    web_sys::console::log_1(
                                        &format!("Successfully deserialized TaskDto: {}", task_dto.name).into()
                                    );
                                    match task_dto.to_task() {
                                        Ok(new_task) => {
                                            web_sys::console::log_1(
                                                &format!("Successfully created task: {}", new_task.name).into()
                                            );
                                            set_is_creating.set(false);
                                        }
                                        Err(e) => {
                                            web_sys::console::error_1(
                                                &format!("Failed to convert TaskDto to Task: {}", e).into()
                                            );
                                            set_is_creating.set(false);
                                        }
                                    }
                                }
                                Err(e) => {
                                    web_sys::console::error_1(
                                        &format!("Failed to deserialize TaskDto: {:?}", e).into()
                                    );
                                    set_is_creating.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to invoke create_task command: {:?}", e).into()
                            );
                            set_is_creating.set(false);
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to serialize args: {:?}", e).into()
                    );
                    set_is_creating.set(false);
                }
            }
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

            if let Ok(args_value) = to_value(&args) {
                web_sys::console::log_1(
                    &format!("Invoking update_task with args: {:?}", args_value).into(),
                );

                match invoke_command(event_names::task::UPDATE, args_value).await {
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
            }
        });
    }

    pub fn cycle_to_next_incomplete_task(&self) {
        let set_current_task = self.set_current_task;
        let set_cycle_position = self.set_cycle_position;
        let current_task = self.current_task;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }

            let current_id = current_task.get_untracked().map(|t| t.id.to_string());

            let args = CycleArgs {
                current_task_id: current_id,
                direction: "next".to_string(),
            };

            if let Ok(args_value) = to_value(&args) {
                if let Ok(result) = invoke_command(
                    event_names::task::CYCLE_INCOMPLETE_TASK,
                    args_value,
                )
                .await
                {
                    #[derive(serde::Deserialize)]
                    struct CycleResult {
                        task: Option<Task>,
                        position: usize,
                        total_incomplete: usize,
                    }

                    if let Ok(cycle_result) = from_value::<CycleResult>(result) {
                        set_current_task.set(cycle_result.task);
                        set_cycle_position.set((
                            cycle_result.position,
                            cycle_result.total_incomplete,
                        ));
                    }
                }
            }
        });
    }

    pub fn cycle_to_previous_incomplete_task(&self) {
        let set_current_task = self.set_current_task;
        let set_cycle_position = self.set_cycle_position;
        let current_task = self.current_task;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }

            let current_id = current_task.get_untracked().map(|t| t.id.to_string());

            let args = CycleArgs {
                current_task_id: current_id,
                direction: "previous".to_string(),
            };

            if let Ok(args_value) = to_value(&args) {
                if let Ok(result) = invoke_command(
                    event_names::task::CYCLE_INCOMPLETE_TASK,
                    args_value,
                )
                .await
                {
                    #[derive(serde::Deserialize)]
                    struct CycleResult {
                        task: Option<Task>,
                        position: usize,
                        total_incomplete: usize,
                    }

                    if let Ok(cycle_result) = from_value::<CycleResult>(result) {
                        set_current_task.set(cycle_result.task);
                        set_cycle_position.set((
                            cycle_result.position,
                            cycle_result.total_incomplete,
                        ));
                    }
                }
            }
        });
    }

    pub fn get_cycle_position(&self) -> (usize, usize) {
        self.cycle_position.get()
    }

    pub fn update_cycle_position(&self) {
        let set_cycle_position = self.set_cycle_position;
        let current_task = self.current_task;

        spawn_local(async move {
            if let Some(task) = current_task.get_untracked() {
                #[derive(serde::Serialize)]
                struct GetPositionArgs {
                    task_id: String,
                }

                let args = GetPositionArgs {
                    task_id: task.id.to_string(),
                };

                if let Ok(args_value) = to_value(&args) {
                    if let Ok(result) = invoke_command(
                        event_names::task::GET_TASK_CYCLE_POSITION,
                        args_value,
                    )
                    .await
                    {
                        if let Ok((position, total)) = from_value::<(usize, usize)>(result) {
                            set_cycle_position.set((position, total));
                        }
                    }
                }
            }
        });
    }
}