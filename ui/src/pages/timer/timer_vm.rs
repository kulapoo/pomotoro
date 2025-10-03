use crate::components::{ErrorInfo, handle_command_error};
use domain::event_names::{ui_listeners::timer as timer_event_names, commands};
use domain::{
    Phase, Task, Timer, TimerConfiguration, TimerState, TimerStatus, TimerTick
};
use js_sys;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

use crate::utils::{ViewModel, invoke};

use super::task_ops;

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

// Accessors
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

    pub fn get_active_entity_id(&self) -> Option<String> {
        self.active_task.get().map(|task| task.id.to_string())
    }
}

// Initialization & Event Listeners
impl TimerViewModel {
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
            Self::setup_timer_status_changed_listener(
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
            invoke::<Timer, ()>(commands::timer::GET_STATE, None).await
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok()
                .and_then(|timer| {
                    // Try extracting from timer object first

                    set_timer_state.set(timer.state().clone());
                    let active_task_id = timer.active_task_id().map(|id| id.to_string()).unwrap_or_default();

                    spawn_local(async move {
                        task_ops::fetch_task_by_id(&active_task_id, set_active_task).await;
                    });

                    Some(())
                });
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

    fn setup_timer_status_changed_listener(
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
                        task_ops::fetch_active_task(set_active_task_clone).await;
                    });
                }
            });

            listen(timer_event_names::STATUS_CHANGED, &callback).await;

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
                    &format!("Timer tae tae status changed: {:?}", payload).into(),
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
}

// Timer Commands
impl TimerViewModel {
    pub fn start_pause_timer(&self) {
        let current_state = self.timer_state.get_untracked();
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;
        let active_task = self.active_task.get_untracked();

        spawn_local(async move {

            let command = if current_state.is_running() {
                commands::timer::PAUSE
            }
            else if current_state.is_paused() {
                commands::timer::RESUME
            } else {
                commands::timer::START
            };

            web_sys::console::log_1(
                &format!("Executing timer command: {} (current state: {:?})",
                        command, current_state.status()).into()
            );

            #[derive(serde::Serialize)]
            struct TimerStateArgs {
                remaining_seconds: u32,
            }

            let timer_state_args = TimerStateArgs {
                remaining_seconds: current_state.remaining_seconds(),
            };

            invoke::<(), TimerStateArgs>(commands::timer::UPDATE_TIMER_SECS, Some(timer_state_args)).await
            .map_err(|e| handle_command_error(e, set_error_state))
            .ok();

            #[derive(serde::Serialize)]
            struct TimerArgs {
                task_id: Option<String>,
            }

            let args = TimerArgs {
                task_id: active_task.map(|t| t.id.to_string()),
            };

            invoke::<Timer, TimerArgs>(command, Some(args)).await
            .map(|mut timer| {
                let status = timer.state().status();
                let remaining_seconds = timer.state().remaining_seconds();

                if command == commands::timer::RESUME {
                    timer.set_remaining_seconds(remaining_seconds - 1);
                }

                web_sys::console::log_1(
                    &format!("Timer updated after {}: {:?}", command, timer).into()
                );
                set_timer_state.set(timer.state().clone());
                web_sys::console::log_1(
                    &format!("Timer state updated after {}: {:?}", command, status).into()
                );
            })
            .map_err(|e| handle_command_error(e, set_error_state))
            .ok();
        });
    }

    pub fn reset_timer(&self) {
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            invoke::<Timer, ()>(commands::timer::RESET, None).await
                .map(|timer| set_timer_state.set(timer.state().clone()))
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok();
        });
    }

    pub fn complete_phase(&self) {
        web_sys::console::log_1(&"Phase completion is handled automatically by the backend".into());
    }

    pub fn skip_phase(&self) {
        let set_timer_state = self.set_timer_state;
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            invoke::<Timer, ()>(commands::timer::SKIP_PHASE, None).await
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok()
                .map(|timer| {
                    set_timer_state.set(timer.state().clone());

                    if let Some(task_id) = timer.active_task_id() {
                        let task_id_str = task_id.to_string();
                        spawn_local(async move {
                            task_ops::fetch_task_by_id(&task_id_str, set_active_task).await;
                        });
                    } else {
                        set_active_task.set(None);
                    }
                });
        });
    }
}

// Display & Formatting
impl TimerViewModel {
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
        let active_task = self.active_task.get();
        let mut seconds = state.remaining_seconds();
        let mut minutes = seconds / 60;
        let mut secs = seconds % 60;

        if state == TimerState::Idle {
            seconds = active_task.map(|t| t.config.timer.work_duration.as_secs() as u32).unwrap_or_default();
            minutes = seconds / 60;
            secs = seconds % 60;
        }

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
}

// Internal Utilities
impl TimerViewModel {
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
}
