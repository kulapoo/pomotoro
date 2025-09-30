use crate::utils::{ViewModel, invoke_command, invoke_command_no_args};
use chrono::{DateTime, Utc};
use domain::event_names::ui_listeners::task as task_event_names;
use domain::{Config, Task, TaskId, TaskStatus, TimerConfiguration, TimerState, event_names};
use js_sys;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
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

// DTO to match backend's TaskDto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub current_sessions: u8,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Config>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub default: bool,
}

impl TaskDto {
    // Convert TaskDto to domain Task
    pub fn to_task(&self) -> Result<Task, String> {
        let task_id = TaskId::from_string(&self.id)
            .map_err(|e| format!("Invalid task ID: {}", e))?;

        let status = match self.status.as_str() {
            "Active" | "active" => TaskStatus::Active,
            "Completed" | "completed" => TaskStatus::Completed,
            "Paused" | "paused" => TaskStatus::Paused,
            "Queued" | "queued" => TaskStatus::Queued,
            _ => TaskStatus::Queued,
        };

        let config = self.settings.clone().unwrap_or_default();

        Ok(Task {
            id: task_id,
            name: self.name.clone(),
            description: self.description.clone(),
            max_sessions: self.max_sessions,
            current_sessions: self.current_sessions,
            tags: self.tags.clone(),
            config,
            created_at: self.created_at,
            updated_at: self.created_at,
            completed_at: self.completed_at,
            status,
            default: self.default,
        })
    }
}

pub struct TasksViewModel {
    tasks: ReadSignal<Vec<Task>>,
    set_tasks: WriteSignal<Vec<Task>>,
    filtered_tasks: ReadSignal<Vec<Task>>,
    set_filtered_tasks: WriteSignal<Vec<Task>>,
    active_task: ReadSignal<Option<Task>>,
    set_active_task: WriteSignal<Option<Task>>,
    selected_task: ReadSignal<Option<TaskId>>,
    set_selected_task: WriteSignal<Option<TaskId>>,
    is_creating: ReadSignal<bool>,
    set_is_creating: WriteSignal<bool>,
    search_query: ReadSignal<String>,
    set_search_query: WriteSignal<String>,
    sort_by: ReadSignal<String>,
    set_sort_by: WriteSignal<String>,
    status_filter: ReadSignal<String>,
    set_status_filter: WriteSignal<String>,
    cycle_position: ReadSignal<(usize, usize)>,
    set_cycle_position: WriteSignal<(usize, usize)>,
}

impl ViewModel for TasksViewModel {
    type State = Vec<Task>;

    fn new() -> Self {
        let (tasks, set_tasks) = signal(Vec::<Task>::new());
        let (filtered_tasks, set_filtered_tasks) = signal(Vec::<Task>::new());
        let (active_task, set_active_task) = signal(None::<Task>);
        let (selected_task, set_selected_task) = signal(None::<TaskId>);
        let (is_creating, set_is_creating) = signal(false);
        let (search_query, set_search_query) = signal(String::new());
        let (sort_by, set_sort_by) = signal("created_at".to_string());
        let (status_filter, set_status_filter) = signal("all".to_string());
        let (cycle_position, set_cycle_position) = signal((0, 0));

        let vm = Self {
            tasks,
            set_tasks,
            filtered_tasks,
            set_filtered_tasks,
            active_task,
            set_active_task,
            selected_task,
            set_selected_task,
            is_creating,
            set_is_creating,
            search_query,
            set_search_query,
            sort_by,
            set_sort_by,
            status_filter,
            set_status_filter,
            cycle_position,
            set_cycle_position,
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

// Helper function to refetch all tasks
async fn refetch_all_tasks(set_tasks: WriteSignal<Vec<Task>>, command: &str) {
    if let Ok(result) = invoke_command_no_args(command).await {
        if let Ok(task_dto_list) = from_value::<Vec<TaskDto>>(result) {
            let mut tasks = Vec::new();
            for dto in task_dto_list {
                if let Ok(task) = dto.to_task() {
                    tasks.push(task);
                }
            }
            set_tasks.set(tasks);
        }
    }
}

impl TasksViewModel {
    fn setup_event_listeners(&self) {
        let set_tasks = self.set_tasks;
        let set_active_task = self.set_active_task;

        // Listen for TaskCreated event
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskCreated event received: {:?}", payload)
                        .into(),
                );

                // Refetch all tasks when a new task is created
                let set_tasks_clone = set_tasks;
                spawn_local(async move {
                    refetch_all_tasks(
                        set_tasks_clone,
                        event_names::task::GET_ALL,
                    )
                    .await;
                });
            });

            listen(event_names::task::TASK_CREATED, &callback).await;
            callback.forget();
        });

        // Listen for TaskUpdated event
        let set_tasks_for_update = self.set_tasks;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskUpdated event received: {:?}", payload)
                        .into(),
                );

                // Refetch all tasks when a task is updated
                let set_tasks_clone = set_tasks_for_update;
                spawn_local(async move {
                    refetch_all_tasks(
                        set_tasks_clone,
                        event_names::task::GET_ALL,
                    )
                    .await;
                });
            });

            listen(event_names::task::TASK_UPDATED, &callback).await;
            callback.forget();
        });

        // Listen for TaskDeleted event
        let set_tasks_for_delete = self.set_tasks;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskDeleted event received: {:?}", payload)
                        .into(),
                );

                // Refetch all tasks when a task is deleted
                let set_tasks_clone = set_tasks_for_delete;
                spawn_local(async move {
                    refetch_all_tasks(
                        set_tasks_clone,
                        event_names::task::GET_ALL,
                    )
                    .await;
                });
            });

            listen(event_names::task::TASK_DELETED, &callback).await;
            callback.forget();
        });

        // Listen for TaskCompleted event
        let set_tasks_for_complete = self.set_tasks;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("TaskCompleted event received: {:?}", payload)
                        .into(),
                );

                // Refetch all tasks when a task is completed
                let set_tasks_clone = set_tasks_for_complete;
                spawn_local(async move {
                    refetch_all_tasks(
                        set_tasks_clone,
                        event_names::task::GET_ALL,
                    )
                    .await;
                });
            });

            listen(event_names::task::TASK_COMPLETED, &callback).await;
            callback.forget();
        });

        // Listen for active task changes
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

                // Try to parse the new active task from the event
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
                    &format!(
                        "Task progress updated event received: {:?}",
                        payload
                    )
                    .into(),
                );

                // Update specific task in the list if provided
                if let Ok(task_dto) = from_value::<TaskDto>(payload.clone()) {
                    if let Ok(updated_task) = task_dto.to_task() {
                        set_tasks_for_progress.update(|tasks| {
                            if let Some(index) = tasks
                                .iter()
                                .position(|t| t.id == updated_task.id)
                            {
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

        spawn_local(async move {
            if let Ok(result) =
                invoke_command_no_args(event_names::task::GET_ALL).await
            {
                // Handle TaskDto list from backend
                if let Ok(task_dto_list) = from_value::<Vec<TaskDto>>(result) {
                    let mut tasks = Vec::new();
                    for dto in task_dto_list {
                        if let Ok(task) = dto.to_task() {
                            tasks.push(task);
                        }
                    }
                    set_tasks.set(tasks);
                }
            }

            if let Ok(result) =
                invoke_command_no_args(event_names::timer::GET_STATE).await
            {
                if let Ok(_timer_state) = from_value::<TimerState>(result) {
                    // In the new architecture, we don't have active_entity_id
                    // Skip this for now
                    if false {
                        if let Ok(task_id) = TaskId::from_string(&"placeholder")
                        {
                            #[derive(serde::Serialize)]
                            struct GetTaskArgs {
                                id: String,
                            }

                            let args = GetTaskArgs {
                                id: task_id.to_string(),
                            };

                            if let Ok(task_args) = to_value(&args) {
                                if let Ok(task_result) = invoke_command(
                                    event_names::task::GET,
                                    task_args,
                                )
                                .await
                                {
                                    // Try to deserialize as TaskDto first
                                    if let Ok(task_dto) = from_value::<TaskDto>(
                                        task_result.clone(),
                                    ) {
                                        if let Ok(task) = task_dto.to_task() {
                                            set_active_task.set(Some(task));
                                        }
                                    } else if let Ok(task) =
                                        from_value::<Task>(task_result)
                                    {
                                        // Fallback to direct Task deserialization
                                        set_active_task.set(Some(task));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        if self.search_query.get().is_empty()
            && self.status_filter.get() == "all"
        {
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

    pub fn is_creating_task(&self) -> bool {
        self.is_creating.get()
    }

    pub fn set_creating_task(&self, creating: bool) {
        self.set_is_creating.set(creating);
    }

    pub fn select_task(&self, task_id: Option<TaskId>) {
        self.set_selected_task.set(task_id);
    }

    pub fn create_task_full(
        &self,
        name: String,
        description: Option<String>,
        max_sessions: usize,
        tags: Vec<String>,
        custom_config: Option<TimerConfiguration>,
    ) {
        let set_tasks = self.set_tasks;
        let set_is_creating = self.set_is_creating;
        let tasks = self.tasks;

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
                        &format!(
                            "Invoking create_task with args: {:?}",
                            args_value
                        )
                        .into(),
                    );
                    match invoke_command(event_names::task::CREATE, args_value)
                        .await
                    {
                        Ok(result) => {
                            web_sys::console::log_1(
                                &format!("Create task result: {:?}", result)
                                    .into(),
                            );
                            // First try to deserialize as TaskDto
                            match from_value::<TaskDto>(result.clone()) {
                                Ok(task_dto) => {
                                    web_sys::console::log_1(&format!("Successfully deserialized TaskDto: {}", task_dto.name).into());
                                    // Convert TaskDto to Task
                                    match task_dto.to_task() {
                                        Ok(new_task) => {
                                            web_sys::console::log_1(&format!("Successfully created task: {}", new_task.name).into());
                                            let mut current_tasks =
                                                tasks.get_untracked();
                                            current_tasks.push(new_task);
                                            set_tasks.set(current_tasks);
                                            set_is_creating.set(false);
                                        }
                                        Err(e) => {
                                            web_sys::console::error_1(&format!("Failed to convert TaskDto to Task: {}", e).into());
                                            // Still refetch to ensure consistency
                                            refetch_all_tasks(
                                                set_tasks,
                                                event_names::task::GET_ALL,
                                            )
                                            .await;
                                            set_is_creating.set(false);
                                        }
                                    }
                                }
                                Err(e) => {
                                    web_sys::console::error_1(&format!("Failed to deserialize TaskDto. Result was: {:?}, Error: {:?}", result, e).into());
                                    // Try to refetch all tasks to ensure we have the latest list
                                    refetch_all_tasks(
                                        set_tasks,
                                        event_names::task::GET_ALL,
                                    )
                                    .await;
                                    set_is_creating.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Failed to invoke create_task command: {:?}", e).into());
                            set_is_creating.set(false);
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to serialize args: {:?}", e).into(),
                    );
                    set_is_creating.set(false);
                }
            }
        });
    }

    pub fn create_task(&self, name: String, description: String) {
        let set_tasks = self.set_tasks;
        let set_is_creating = self.set_is_creating;
        let tasks = self.tasks;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CreateTaskRequest {
                name: String,
                description: Option<String>,
                max_sessions: u8,
                tags: Vec<String>,
                audio_config: Option<domain::AudioConfig>,
            }

            #[derive(serde::Serialize)]
            struct CreateTaskArgs {
                request: CreateTaskRequest,
            }

            let request = CreateTaskRequest {
                name: name.clone(),
                description: if description.is_empty() {
                    None
                } else {
                    Some(description)
                },
                max_sessions: 4, // Default value
                tags: Vec::new(),
                audio_config: None,
            };

            let args = CreateTaskArgs { request };

            match to_value(&args) {
                Ok(args_value) => {
                    web_sys::console::log_1(
                        &format!(
                            "Invoking create_task with args: {:?}",
                            args_value
                        )
                        .into(),
                    );
                    match invoke_command(event_names::task::CREATE, args_value)
                        .await
                    {
                        Ok(result) => {
                            web_sys::console::log_1(
                                &format!("Create task result: {:?}", result)
                                    .into(),
                            );
                            // First try to deserialize as TaskDto
                            match from_value::<TaskDto>(result.clone()) {
                                Ok(task_dto) => {
                                    web_sys::console::log_1(&format!("Successfully deserialized TaskDto: {}", task_dto.name).into());
                                    // Convert TaskDto to Task
                                    match task_dto.to_task() {
                                        Ok(new_task) => {
                                            web_sys::console::log_1(&format!("Successfully created task: {}", new_task.name).into());
                                            let mut current_tasks =
                                                tasks.get_untracked();
                                            current_tasks.push(new_task);
                                            set_tasks.set(current_tasks);
                                            set_is_creating.set(false);
                                        }
                                        Err(e) => {
                                            web_sys::console::error_1(&format!("Failed to convert TaskDto to Task: {}", e).into());
                                            // Still refetch to ensure consistency
                                            refetch_all_tasks(
                                                set_tasks,
                                                event_names::task::GET_ALL,
                                            )
                                            .await;
                                            set_is_creating.set(false);
                                        }
                                    }
                                }
                                Err(e) => {
                                    web_sys::console::error_1(&format!("Failed to deserialize TaskDto. Result was: {:?}, Error: {:?}", result, e).into());
                                    // Try to refetch all tasks to ensure we have the latest list
                                    refetch_all_tasks(
                                        set_tasks,
                                        event_names::task::GET_ALL,
                                    )
                                    .await;
                                    set_is_creating.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Failed to invoke create_task command: {:?}", e).into());
                            set_is_creating.set(false);
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to serialize args: {:?}", e).into(),
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
        let set_tasks = self.set_tasks;
        let tasks = self.tasks;

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
                    &format!(
                        "Invoking update_task with args: {:?}",
                        args_value
                    )
                    .into(),
                );

                match invoke_command(event_names::task::UPDATE, args_value).await {
                    Ok(result) => {
                        web_sys::console::log_1(
                            &format!("Update task result: {:?}", result)
                                .into(),
                        );

                        // Try to deserialize as TaskDto first
                        match from_value::<TaskDto>(result.clone()) {
                            Ok(task_dto) => {
                                match task_dto.to_task() {
                                    Ok(updated_task) => {
                                        let mut current_tasks = tasks.get_untracked();
                                        if let Some(index) =
                                            current_tasks.iter().position(|t| t.id == task_id)
                                        {
                                            current_tasks[index] = updated_task;
                                            set_tasks.set(current_tasks);
                                        }
                                    }
                                    Err(e) => {
                                        web_sys::console::error_1(&format!("Failed to convert TaskDto to Task: {}", e).into());
                                        // Still refetch to ensure consistency
                                        refetch_all_tasks(
                                            set_tasks,
                                            event_names::task::GET_ALL,
                                        )
                                        .await;
                                    }
                                }
                            }
                            Err(e) => {
                                web_sys::console::error_1(&format!("Failed to deserialize TaskDto: {:?}", e).into());
                                // Try to refetch all tasks to ensure we have the latest list
                                refetch_all_tasks(
                                    set_tasks,
                                    event_names::task::GET_ALL,
                                )
                                .await;
                            }
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to invoke update_task command: {:?}", e).into());
                    }
                }
            }
        });
    }

    pub fn delete_task(&self, task_id: TaskId) -> bool {
        // Show confirmation dialog
        let task_name = self.tasks
            .get()
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "this task".to_string());

        let confirmed = leptos::prelude::window()
            .confirm_with_message(&format!("Are you sure you want to delete \"{}\"?", task_name))
            .unwrap_or(false);
        web_sys::console::log_1(
            &format!("Confirmed: {:?}", confirmed).into(),
        );
        if !confirmed {
            return false;
        }

        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_selected_task = self.set_selected_task;

        spawn_local(async move {
            #[derive(Serialize)]
            struct DeleteTaskArgs {
                id: String,
            }

            let args = DeleteTaskArgs {
                id: task_id.to_string(),
            };

            if let Ok(args_value) = to_value(&args) {
                web_sys::console::log_1(
                    &format!(
                        "Invoking delete_task with args: {:?}",
                        args_value
                    )
                    .into(),
                );

                match invoke_command(event_names::task::DELETE, args_value).await {
                    Ok(_result) => {
                        web_sys::console::log_1(
                            &format!("Successfully deleted task: {:?}", task_id)
                                .into(),
                        );
                        let mut current_tasks = tasks.get_untracked();
                        current_tasks.retain(|t| t.id != task_id);
                        set_tasks.set(current_tasks);
                        set_selected_task.set(None);
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to delete task: {:?}", e).into());
                    }
                }
            }
        });

        true
    }

    pub fn switch_active_task(&self, task_id: TaskId) {
        let set_active_task = self.set_active_task;
        let tasks = self.tasks;

        spawn_local(async move {
            // Create a JS object with taskId as the key
            // Tauri expects parameters at the top level
            web_sys::console::log_1(
                &format!("Switching to task: {:?}", task_id).into(),
            );

            let args_obj = js_sys::Object::new();

            // Serialize the TaskId and set it as the taskId property
            if let Ok(task_id_value) = to_value(&task_id) {
                js_sys::Reflect::set(
                    &args_obj,
                    &JsValue::from_str("taskId"),
                    &task_id_value,
                )
                .unwrap();

                web_sys::console::log_1(
                    &format!(
                        "Invoking switch_active_task with args: {:?}",
                        args_obj
                    )
                    .into(),
                );

                match invoke_command(
                    event_names::timer::SWITCH_ACTIVE_TASK,
                    args_obj.into(),
                )
                .await
                {
                    Ok(result) => {
                        web_sys::console::log_1(
                            &format!("Switch task result: {:?}", result).into(),
                        );
                        if let Ok(_timer_state) =
                            from_value::<TimerState>(result)
                        {
                            // In the new architecture, switching means working with a specific task's timer
                            // Use the task_id we just switched to
                            {
                                let active_id = task_id;
                                let task_list = tasks.get_untracked();
                                let active_task = task_list
                                    .iter()
                                    .find(|t| t.id == active_id)
                                    .cloned();
                                let task_name = active_task
                                    .as_ref()
                                    .map(|t| t.name.clone());
                                set_active_task.set(active_task);
                                web_sys::console::log_1(
                                    &format!(
                                        "Active task set to: {:?}",
                                        task_name
                                    )
                                    .into(),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to switch task: {:?}", e).into(),
                        );
                    }
                }
            } else {
                web_sys::console::error_1(
                    &"Failed to serialize task ID".into(),
                );
            }
        });
    }

    pub fn refetch_tasks(&self) {
        let set_tasks = self.set_tasks;
        let set_active_task = self.set_active_task;
        let tasks = self.tasks;

        spawn_local(async move {
            if let Ok(result) =
                invoke_command_no_args(event_names::task::GET_ALL).await
            {
                // Handle TaskDto list from backend
                if let Ok(task_dto_list) = from_value::<Vec<TaskDto>>(result) {
                    let mut task_list = Vec::new();
                    for dto in task_dto_list {
                        if let Ok(task) = dto.to_task() {
                            task_list.push(task);
                        }
                    }
                    set_tasks.set(task_list);
                }
            }

            if let Ok(result) =
                invoke_command_no_args(event_names::timer::GET_STATE).await
            {
                if let Ok(_timer_state) = from_value::<TimerState>(result) {
                    // In the new architecture, we don't have active_entity_id
                    // Skip this for now
                    if false {
                        if let Ok(task_id) = TaskId::from_string(&"placeholder")
                        {
                            let task_list = tasks.get_untracked();
                            let active_task = task_list
                                .iter()
                                .find(|t| t.id == task_id)
                                .cloned();
                            set_active_task.set(active_task);
                        }
                    } else {
                        set_active_task.set(None);
                    }
                }
            }
        });
    }

    pub fn search_tasks(&self, query: String) {
        self.set_search_query.set(query.clone());

        if query.is_empty() && self.status_filter.get() == "all" {
            self.set_filtered_tasks.set(self.tasks.get());
            return;
        }

        let set_filtered = self.set_filtered_tasks;
        let sort_by = self.sort_by.get();
        let status_filter = self.status_filter.get();

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct SearchArgs {
                query: Option<String>,
                status: Option<String>,
                sort_by: Option<String>,
                sort_order: Option<String>,
            }

            let args = SearchArgs {
                query: if query.is_empty() { None } else { Some(query) },
                status: if status_filter == "all" {
                    None
                } else {
                    Some(status_filter)
                },
                sort_by: Some(sort_by),
                sort_order: Some("asc".to_string()),
            };

            if let Ok(args_value) = to_value(&args) {
                if let Ok(result) =
                    invoke_command(event_names::task::SEARCH, args_value).await
                {
                    if let Ok(task_list) = from_value::<Vec<Task>>(result) {
                        set_filtered.set(task_list);
                    }
                }
            }
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

    pub fn get_search_query(&self) -> String {
        self.search_query.get()
    }

    pub fn get_sort_by(&self) -> String {
        self.sort_by.get()
    }

    pub fn get_status_filter(&self) -> String {
        self.status_filter.get()
    }

    pub fn cycle_to_next_incomplete_task(&self) {
        let set_active_task = self.set_active_task;
        let set_cycle_position = self.set_cycle_position;
        let active_task = self.active_task;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }

            let current_id =
                active_task.get_untracked().map(|t| t.id.to_string());

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

                    if let Ok(cycle_result) = from_value::<CycleResult>(result)
                    {
                        set_active_task.set(cycle_result.task);
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
        let set_active_task = self.set_active_task;
        let set_cycle_position = self.set_cycle_position;
        let active_task = self.active_task;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }

            let current_id =
                active_task.get_untracked().map(|t| t.id.to_string());

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

                    if let Ok(cycle_result) = from_value::<CycleResult>(result)
                    {
                        set_active_task.set(cycle_result.task);
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
        let active_task = self.active_task;

        spawn_local(async move {
            if let Some(task) = active_task.get_untracked() {
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
                        if let Ok((position, total)) =
                            from_value::<(usize, usize)>(result)
                        {
                            set_cycle_position.set((position, total));
                        }
                    }
                }
            }
        });
    }
}
