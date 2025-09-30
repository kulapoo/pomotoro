use crate::components::{ErrorInfo, handle_command_error};
use domain::event_names::ui_listeners::timer as timer_event_names;
use domain::{
    Phase, Task, TaskId, TimerConfiguration, TimerState, TimerStatus,
    TimerTick,
};
use js_sys;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

use crate::utils::{ViewModel, invoke_command, invoke_command_no_args};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(
        event: &str,
        callback: &Closure<dyn Fn(JsValue)>,
    ) -> JsValue;
}

pub struct TimerViewModel {
    timer_state: ReadSignal<TimerState>,
    set_timer_state: WriteSignal<TimerState>,
    timer_config: ReadSignal<TimerConfiguration>,
    set_timer_config: WriteSignal<TimerConfiguration>,
    active_task: ReadSignal<Option<Task>>,
    set_active_task: WriteSignal<Option<Task>>,
    error_state: ReadSignal<Option<ErrorInfo>>,
    set_error_state: WriteSignal<Option<ErrorInfo>>,
}

impl ViewModel for TimerViewModel {
    type State = TimerState;

    fn new() -> Self {
        let (timer_state, set_timer_state) = signal(TimerState::default());
        let (timer_config, set_timer_config) =
            signal(TimerConfiguration::default());
        let (active_task, set_active_task) = signal(None::<Task>);
        let (error_state, set_error_state) = signal(None::<ErrorInfo>);

        let vm = Self {
            timer_state,
            set_timer_state,
            timer_config,
            set_timer_config,
            active_task,
            set_active_task,
            error_state,
            set_error_state,
        };

        vm.initialize();
        vm
    }

    fn state(&self) -> ReadSignal<Self::State> {
        self.timer_state
    }

    fn set_state(&self) -> WriteSignal<Self::State> {
        self.set_timer_state
    }
}

impl TimerViewModel {
    pub fn error_state(&self) -> ReadSignal<Option<ErrorInfo>> {
        self.error_state
    }

    pub fn set_error_state(&self) -> WriteSignal<Option<ErrorInfo>> {
        self.set_error_state
    }

    pub fn get_active_task(&self) -> Option<Task> {
        self.active_task.get()
    }

    pub fn get_active_task_name(&self) -> String {
        self.active_task
            .get()
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "No Task Selected".to_string())
    }

    fn initialize(&self) {
        let set_timer_state = self.set_timer_state;
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;

        Effect::new(move |_| {
            Self::setup_initial_state(
                set_timer_state,
                set_active_task,
                set_error_state,
            );
            Self::setup_timer_tick_listener(set_timer_state);
            Self::setup_timer_state_updated_listener(
                set_timer_state,
                set_active_task,
            );
            Self::setup_phase_event_listeners(set_timer_state, set_active_task);
        });
    }

    fn setup_initial_state(
        set_timer_state: WriteSignal<TimerState>,
        set_active_task: WriteSignal<Option<Task>>,
        set_error_state: WriteSignal<Option<ErrorInfo>>,
    ) {
        spawn_local(async move {
            // Use the actual Tauri command name
            match invoke_command_no_args("get_timer_state").await {
                Ok(result) => {
                    // Now the result is TimerInfo containing both state and active_task_id
                    if let Ok(timer_info) = serde_wasm_bindgen::from_value::<serde_json::Value>(result.clone()) {
                        // Extract timer state
                        if let Some(state_val) = timer_info.get("state") {
                            if let Ok(state) = serde_json::from_value::<TimerState>(state_val.clone()) {
                                set_timer_state.set(state.clone());
                            }
                        }

                        // Extract active task ID and fetch the task
                        if let Some(active_task_id) = timer_info.get("active_task_id") {
                            if !active_task_id.is_null() {
                                if let Some(task_id_str) = active_task_id.as_str() {
                                    // Fetch the specific task by ID
                                    Self::fetch_task_by_id(task_id_str, set_active_task).await;
                                } else {
                                    set_active_task.set(None);
                                }
                            } else {
                                set_active_task.set(None);
                            }
                        } else {
                            set_active_task.set(None);
                        }
                    } else if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                        // Fallback: try parsing as just TimerState for backwards compatibility
                        set_timer_state.set(state.clone());
                        Self::fetch_active_task(set_active_task).await;
                    } else {
                        web_sys::console::error_1(
                            &"Failed to parse initial timer state".into(),
                        );
                    }
                }
                Err(error) => {
                    let error_str = format!(
                        "Failed to get initial timer state: {:?}",
                        error
                    );
                    web_sys::console::error_1(&error_str.clone().into());
                    handle_command_error(error_str, set_error_state);
                }
            }
        });
    }

    fn setup_timer_tick_listener(set_timer_state: WriteSignal<TimerState>) {
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                if let Ok(timer_tick) =
                    serde_wasm_bindgen::from_value::<TimerTick>(payload)
                {
                    set_timer_state.update(|state| {
                        *state = state.with_remaining_seconds(
                            timer_tick.remaining_seconds,
                        );
                    });
                }
            });

            listen(timer_event_names::TICK, &callback).await;

            callback.forget();
        });
    }

    fn setup_timer_state_updated_listener(
        set_timer_state: WriteSignal<TimerState>,
        set_active_task: WriteSignal<Option<Task>>,
    ) {
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                if let Ok(state) =
                    serde_wasm_bindgen::from_value::<TimerState>(payload)
                {
                    set_timer_state.set(state.clone());

                    // Fetch updated active task info
                    let set_active_task_clone = set_active_task;
                    spawn_local(async move {
                        Self::fetch_active_task(set_active_task_clone).await;
                    });
                }
            });

            listen(timer_event_names::STATE_UPDATED, &callback).await;

            callback.forget();
        });
    }

    fn setup_phase_event_listeners(
        set_timer_state: WriteSignal<TimerState>,
        _set_active_task: WriteSignal<Option<Task>>,
    ) {
        // Listen for phase completed events
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                // Log phase completion for debugging
                web_sys::console::log_1(
                    &format!("Phase completed event received: {:?}", payload)
                        .into(),
                );

                // Update timer state if provided in the event
                if let Ok(state) =
                    serde_wasm_bindgen::from_value::<TimerState>(payload)
                {
                    set_timer_state.set(state);
                }
            });

            listen(timer_event_names::PHASE_COMPLETED, &callback).await;
            callback.forget();
        });

        // Listen for phase skipped events
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                // Log phase skip for debugging
                web_sys::console::log_1(
                    &format!("Phase skipped event received: {:?}", payload)
                        .into(),
                );

                // Update timer state if provided in the event
                if let Ok(state) =
                    serde_wasm_bindgen::from_value::<TimerState>(payload)
                {
                    set_timer_state.set(state);
                }
            });

            listen(timer_event_names::PHASE_SKIPPED, &callback).await;
            callback.forget();
        });

        // Listen for status changed events
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                // Log status change for debugging
                web_sys::console::log_1(
                    &format!("Timer status changed: {:?}", payload).into(),
                );

                // Update timer state if provided in the event
                if let Ok(state) =
                    serde_wasm_bindgen::from_value::<TimerState>(payload)
                {
                    set_timer_state.set(state);
                }
            });

            listen(timer_event_names::STATUS_CHANGED, &callback).await;
            callback.forget();
        });
    }

    async fn fetch_task_details(
        entity_id: String,
        set_active_task: WriteSignal<Option<Task>>,
    ) {
        if let Ok(task_id) = TaskId::from_string(&entity_id) {
            #[derive(serde::Serialize)]
            struct GetTaskArgs {
                id: String,
            }

            let args = GetTaskArgs {
                id: task_id.to_string(),
            };

            if let Ok(args_value) = serde_wasm_bindgen::to_value(&args) {
                // Use the actual Tauri command name
                if let Ok(result) =
                    invoke_command("get_task", args_value).await
                {
                    // Try to parse the TaskDto from task_vm.rs
                    if let Ok(task_dto) = serde_wasm_bindgen::from_value::<
                        crate::pages::task::TaskDto,
                    >(result.clone())
                    {
                        if let Ok(task) = task_dto.to_task() {
                            set_active_task.set(Some(task));
                        }
                    } else if let Ok(task) =
                        serde_wasm_bindgen::from_value::<Task>(result)
                    {
                        set_active_task.set(Some(task));
                    }
                }
            }
        }
    }

    pub fn start_pause_timer(&self) {
        let current_state = self.timer_state.get_untracked();
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;
        let active_task = self.active_task.get_untracked();

        spawn_local(async move {
            // Determine the correct command based on current state
            let command = if current_state.is_running() {
                // Timer is running, pause it
                "pause_timer"
            } else {
                // Timer is idle or paused, start/resume it
                // The backend start_timer handles resume automatically when paused
                "start_timer"
            };

            // Log the action for debugging
            web_sys::console::log_1(
                &format!("Executing timer command: {} (current state: {:?})",
                        command, current_state.status()).into()
            );

            // For start_timer, we need to pass task_id if available
            let result = if command == "start_timer" && active_task.is_some() {
                #[derive(serde::Serialize)]
                struct StartTimerArgs {
                    task_id: Option<String>,
                }

                let args = StartTimerArgs {
                    task_id: active_task.map(|t| t.id.to_string()),
                };

                if let Ok(args_value) = serde_wasm_bindgen::to_value(&args) {
                    invoke_command(command, args_value).await
                } else {
                    invoke_command_no_args(command).await
                }
            } else {
                invoke_command_no_args(command).await
            };

            match result {
                Ok(result) => {
                    // Try to parse the timer state from the response
                    if let Ok(state) =
                        serde_wasm_bindgen::from_value::<TimerState>(result.clone())
                    {
                        let status = state.status();
                        set_timer_state.set(state);
                        web_sys::console::log_1(
                            &format!("Timer state updated after {}: {:?}",
                                    command, status).into()
                        );
                    } else {
                        // Some commands might not return state directly
                        web_sys::console::log_1(
                            &format!("{} command executed successfully", command).into()
                        );
                    }
                }
                Err(error) => {
                    let error_str =
                        format!("Failed to execute {}: {:?}", command, error);
                    web_sys::console::error_1(&error_str.clone().into());
                    handle_command_error(error_str, set_error_state);
                }
            }
        });
    }

    pub fn reset_timer(&self) {
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            // Use the actual Tauri command name
            match invoke_command_no_args("reset_timer").await {
                Ok(result) => {
                    if let Ok(state) =
                        serde_wasm_bindgen::from_value::<TimerState>(result)
                    {
                        set_timer_state.set(state);
                    } else {
                        web_sys::console::error_1(
                            &"Failed to parse timer state from reset".into(),
                        );
                    }
                }
                Err(error) => {
                    let error_str =
                        format!("Failed to reset timer: {:?}", error);
                    web_sys::console::error_1(&error_str.clone().into());
                    handle_command_error(error_str, set_error_state);
                }
            }
        });
    }

    pub fn complete_phase(&self) {
        // Phase completion happens automatically through the timer tick mechanism
        // This method is kept for compatibility but doesn't need to do anything
        // as the backend timer service handles phase transitions automatically
        web_sys::console::log_1(&"Phase completion is handled automatically by the backend".into());
    }

    pub fn skip_phase(&self) {
        let set_timer_state = self.set_timer_state;
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            // Use the actual Tauri command name
            match invoke_command_no_args("skip_phase").await {
                Ok(result) => {
                    // skip_phase now returns TimerInfo
                    if let Ok(timer_info) = serde_wasm_bindgen::from_value::<serde_json::Value>(result.clone()) {
                        // Extract timer state
                        if let Some(state_val) = timer_info.get("state") {
                            if let Ok(state) = serde_json::from_value::<TimerState>(state_val.clone()) {
                                set_timer_state.set(state);
                            }
                        }

                        // Extract active task ID and fetch the task
                        if let Some(active_task_id) = timer_info.get("active_task_id") {
                            if !active_task_id.is_null() {
                                if let Some(task_id_str) = active_task_id.as_str() {
                                    Self::fetch_task_by_id(task_id_str, set_active_task).await;
                                }
                            }
                        }
                    } else if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                        // Fallback for backwards compatibility
                        set_timer_state.set(state);
                    } else {
                        web_sys::console::error_1(
                            &"Failed to parse timer state from skip_phase".into()
                        );
                    }
                }
                Err(error) => {
                    let error_str =
                        format!("Failed to skip phase: {:?}", error);
                    web_sys::console::error_1(&error_str.clone().into());
                    handle_command_error(error_str, set_error_state);
                }
            }
        });
    }

    pub fn switch_task(&self, task_id: TaskId) {
        let set_timer_state = self.set_timer_state;
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct SwitchTaskArgs {
                task_id: String,
            }

            let args = SwitchTaskArgs {
                task_id: task_id.to_string(),
            };

            if let Ok(args_value) = serde_wasm_bindgen::to_value(&args) {
                // Use the actual Tauri command name
                match invoke_command("switch_active_task", args_value).await {
                    Ok(result) => {
                        // Now the result is TimerInfo containing both state and active_task_id
                        if let Ok(timer_info) = serde_wasm_bindgen::from_value::<serde_json::Value>(result.clone()) {
                            // Extract timer state
                            if let Some(state_val) = timer_info.get("state") {
                                if let Ok(state) = serde_json::from_value::<TimerState>(state_val.clone()) {
                                    set_timer_state.set(state);
                                }
                            }

                            // Extract active task ID and fetch the task
                            if let Some(active_task_id) = timer_info.get("active_task_id") {
                                if !active_task_id.is_null() {
                                    if let Some(task_id_str) = active_task_id.as_str() {
                                        Self::fetch_task_by_id(task_id_str, set_active_task).await;
                                    }
                                }
                            }
                        } else if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result.clone()) {
                            // Fallback for backwards compatibility
                            set_timer_state.set(state);
                            Self::fetch_active_task(set_active_task).await;
                        }
                    }
                    Err(error) => {
                        let error_str = format!("Failed to switch task: {:?}", error);
                        web_sys::console::error_1(&error_str.clone().into());
                        handle_command_error(error_str, set_error_state);
                    }
                }
            }
        });
    }

    async fn check_task_cycle(set_active_task: WriteSignal<Option<Task>>) {
        use crate::pages::task::types::TaskDto;

        // Check if current task has reached max sessions and needs to cycle
        // Use the actual Tauri command name
        match invoke_command_no_args("get_active_tasks").await {
            Ok(result) => {
                if let Ok(task_dtos) = serde_wasm_bindgen::from_value::<Vec<TaskDto>>(result) {
                    if let Some(task_dto) = task_dtos.first() {
                        if let Ok(task) = task_dto.to_task() {
                            // Check if task completed its max sessions
                            if task.current_sessions >= task.max_sessions {
                                // Cycle to next incomplete task
                                Self::cycle_to_next_task(set_active_task).await;
                            } else {
                                set_active_task.set(Some(task));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to check task cycle: {:?}", e).into());
            }
        }
    }

    async fn cycle_to_next_task(set_active_task: WriteSignal<Option<Task>>) {
        use crate::pages::task::types::TaskDto;

        // Use the actual Tauri command name
        match invoke_command_no_args("cycle_incomplete_task").await {
            Ok(result) => {
                if let Ok(task_dto) = serde_wasm_bindgen::from_value::<TaskDto>(result.clone()) {
                    if let Ok(task) = task_dto.to_task() {
                        set_active_task.set(Some(task));
                        web_sys::console::log_1(&format!("Cycled to next task").into());
                    }
                } else if let Ok(task) = serde_wasm_bindgen::from_value::<Task>(result) {
                    // Fallback for backwards compatibility
                    set_active_task.set(Some(task));
                    web_sys::console::log_1(&format!("Cycled to next task").into());
                }
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to cycle task: {:?}", e).into());
            }
        }
    }

    async fn fetch_active_task(set_active_task: WriteSignal<Option<Task>>) {
        use crate::pages::task::types::TaskDto;

        // Use the actual Tauri command name
        match invoke_command_no_args("get_active_tasks").await {
            Ok(result) => {
                if let Ok(task_dtos) = serde_wasm_bindgen::from_value::<Vec<TaskDto>>(result) {
                    if let Some(task_dto) = task_dtos.first() {
                        if let Ok(task) = task_dto.to_task() {
                            set_active_task.set(Some(task));
                        } else {
                            set_active_task.set(None);
                        }
                    } else {
                        set_active_task.set(None);
                    }
                } else {
                    set_active_task.set(None);
                }
            }
            Err(_) => {
                set_active_task.set(None);
            }
        }
    }

    async fn fetch_task_by_id(task_id: &str, set_active_task: WriteSignal<Option<Task>>) {
        use serde::Serialize;
        use serde_wasm_bindgen::to_value;
        use crate::pages::task::types::TaskDto;

        #[derive(Serialize)]
        struct GetTaskArgs {
            id: String,
        }

        let args = GetTaskArgs {
            id: task_id.to_string(),
        };

        if let Ok(args_value) = to_value(&args) {
            match invoke_command("get_task", args_value).await {
                Ok(result) => {
                    // get_task returns Option<TaskDto>
                    if let Ok(task_dto_opt) = serde_wasm_bindgen::from_value::<Option<TaskDto>>(result) {
                        if let Some(task_dto) = task_dto_opt {
                            // Convert TaskDto to Task
                            match task_dto.to_task() {
                                Ok(task) => {
                                    web_sys::console::log_1(&format!("Timer page: Loaded active task: {}", task.name).into());
                                    set_active_task.set(Some(task));
                                }
                                Err(e) => {
                                    web_sys::console::error_1(&format!("Timer page: Failed to convert TaskDto to Task: {}", e).into());
                                    set_active_task.set(None);
                                }
                            }
                        } else {
                            web_sys::console::log_1(&"Timer page: Task not found".into());
                            set_active_task.set(None);
                        }
                    } else {
                        web_sys::console::error_1(&"Timer page: Failed to parse task response".into());
                        set_active_task.set(None);
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Timer page: Failed to fetch task by ID: {:?}", e).into());
                    set_active_task.set(None);
                }
            }
        }
    }

    pub fn get_phase_name(&self) -> String {
        let state = self.timer_state.get();
        match &state {
            TimerState::Idle { .. } => "Idle".to_string(),
            TimerState::Working { .. } => "Focus Time".to_string(),
            TimerState::ShortBreak { .. } => "Short Break".to_string(),
            TimerState::LongBreak { .. } => "Long Break".to_string(),
            TimerState::Paused { paused_from, .. } => {
                match paused_from.as_ref() {
                    TimerState::Working { .. } => {
                        "Focus Time (Paused)".to_string()
                    }
                    TimerState::ShortBreak { .. } => {
                        "Short Break (Paused)".to_string()
                    }
                    TimerState::LongBreak { .. } => {
                        "Long Break (Paused)".to_string()
                    }
                    _ => "Paused".to_string(),
                }
            }
        }
    }

    pub fn format_time(&self) -> String {
        let state = self.timer_state.get();
        let seconds = state.remaining_seconds();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{minutes:02}:{secs:02}")
    }

    pub fn get_progress_percentage(&self) -> f64 {
        let state = self.timer_state.get();
        let config = self.timer_config.get();
        let remaining = state.remaining_seconds();
        let total = match &state {
            TimerState::Working { .. } => {
                config.get_phase_duration_seconds(Phase::Work)
            }
            TimerState::ShortBreak { .. } => {
                config.get_phase_duration_seconds(Phase::ShortBreak)
            }
            TimerState::LongBreak { .. } => {
                config.get_phase_duration_seconds(Phase::LongBreak)
            }
            TimerState::Paused { paused_from, .. } => {
                let phase = Self::get_current_phase_static(paused_from);
                config.get_phase_duration_seconds(phase)
            }
            TimerState::Idle => return 0.0,
        };

        if total == 0 {
            0.0
        } else {
            ((total - remaining) as f64 / total as f64) * 100.0
        }
    }

    pub fn get_session_display(&self) -> String {
        if let Some(task) = self.active_task.get() {
            let config = self.timer_config.get();
            let sessions_until_long_break =
                config.sessions_until_long_break as u32;
            format!(
                "Session {}/{}",
                (task.current_sessions % sessions_until_long_break as u8) + 1,
                sessions_until_long_break
            )
        } else {
            "No active task".to_string()
        }
    }

    pub fn get_start_pause_button_text(&self) -> &'static str {
        match self.timer_state.get().status() {
            TimerStatus::Running => "Pause",
            _ => "Start",
        }
    }

    fn get_current_phase_static(state: &TimerState) -> Phase {
        match state {
            TimerState::Working { .. } => Phase::Work,
            TimerState::ShortBreak { .. } => Phase::ShortBreak,
            TimerState::LongBreak { .. } => Phase::LongBreak,
            TimerState::Idle { .. } => Phase::Work,
            TimerState::Paused { paused_from, .. } => {
                Self::get_current_phase_static(paused_from)
            }
        }
    }

    pub fn get_sessions_completed(&self) -> usize {
        if let Some(task) = self.active_task.get() {
            let config = self.timer_config.get();
            (task.current_sessions % config.sessions_until_long_break) as usize
        } else {
            0
        }
    }

    pub fn get_today_pomodoros(&self) -> u32 {
        // This would typically come from a stats service, for now return task sessions
        if let Some(task) = self.active_task.get() {
            task.current_sessions as u32
        } else {
            0
        }
    }

    pub fn get_task_pomodoros(&self) -> u32 {
        if let Some(task) = self.active_task.get() {
            task.current_sessions as u32
        } else {
            0
        }
    }

    pub fn get_active_entity_id(&self) -> Option<String> {
        self.active_task.get().map(|task| task.id.to_string())
    }
}
