use crate::components::{ErrorInfo, handle_command_error};
use domain::{Phase, TimerState, TimerStatus, TimerTick, event_names};
use domain::event_names::ui_listeners::timer as timer_event_names;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;
use js_sys;

use crate::utils::{
    ViewModel, invoke_command_no_args
};

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
    error_state: ReadSignal<Option<ErrorInfo>>,
    set_error_state: WriteSignal<Option<ErrorInfo>>,
}

impl ViewModel for TimerViewModel {
    type State = TimerState;

    fn new() -> Self {
        let (timer_state, set_timer_state) = signal(TimerState::default());
        let (error_state, set_error_state) = signal(None::<ErrorInfo>);

        let vm = Self {
            timer_state,
            set_timer_state,
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

    fn initialize(&self) {
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;

        Effect::new(move |_| {
            Self::setup_initial_state(set_timer_state, set_error_state);
            Self::setup_timer_tick_listener(set_timer_state);
        });
    }

    fn setup_initial_state(
        set_timer_state: WriteSignal<TimerState>,
        set_error_state: WriteSignal<Option<ErrorInfo>>,
    ) {
        spawn_local(async move {
            match invoke_command_no_args(event_names::timer::GET_STATE).await {
                Ok(result) => {
                    if let Ok(state) =
                        serde_wasm_bindgen::from_value::<TimerState>(result)
                    {
                        set_timer_state.set(state);
                    } else {
                        web_sys::console::error_1(
                            &"Failed to parse initial timer state".into()
                        );
                    }
                }
                Err(error) => {
                    let error_str = format!("Failed to get initial timer state: {:?}", error);
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
                        // Update the timer state with the new remaining seconds
                        // This ensures the UI stays in sync with the backend timer
                        *state = match state {
                            TimerState::Working { configuration, session_count, active_entity, entity_session_count, .. } => {
                                TimerState::Working {
                                    remaining_seconds: timer_tick.remaining_seconds,
                                    configuration: configuration.clone(),
                                    session_count: *session_count,
                                    active_entity: active_entity.clone(),
                                    entity_session_count: *entity_session_count,
                                }
                            }
                            TimerState::ShortBreak { configuration, session_count, active_entity, entity_session_count, .. } => {
                                TimerState::ShortBreak {
                                    remaining_seconds: timer_tick.remaining_seconds,
                                    configuration: configuration.clone(),
                                    session_count: *session_count,
                                    active_entity: active_entity.clone(),
                                    entity_session_count: *entity_session_count,
                                }
                            }
                            TimerState::LongBreak { configuration, session_count, active_entity, entity_session_count, .. } => {
                                TimerState::LongBreak {
                                    remaining_seconds: timer_tick.remaining_seconds,
                                    configuration: configuration.clone(),
                                    session_count: *session_count,
                                    active_entity: active_entity.clone(),
                                    entity_session_count: *entity_session_count,
                                }
                            }
                            _ => state.clone(),
                        };
                    });
                }
            });

            listen(timer_event_names::TICK, &callback).await;

            callback.forget();
        });
    }

    pub fn start_pause_timer(&self) {
        let current_state = self.timer_state.get_untracked();
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            let command = match current_state.status() {
                TimerStatus::Running => event_names::timer::PAUSE,
                _ => event_names::timer::START,
            };

            match invoke_command_no_args(command).await {
                Ok(result) => {
                    if let Ok(state) =
                        serde_wasm_bindgen::from_value::<TimerState>(result)
                    {
                        set_timer_state.set(state);
                    } else {
                        web_sys::console::error_1(
                            &format!("Failed to parse timer state from {}", command).into()
                        );
                    }
                }
                Err(error) => {
                    let error_str = format!("Failed to execute {}: {:?}", command, error);
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
            match invoke_command_no_args(event_names::timer::RESET).await {
                Ok(result) => {
                    if let Ok(state) =
                        serde_wasm_bindgen::from_value::<TimerState>(result)
                    {
                        set_timer_state.set(state);
                    } else {
                        web_sys::console::error_1(
                            &"Failed to parse timer state from reset".into()
                        );
                    }
                }
                Err(error) => {
                    let error_str = format!("Failed to reset timer: {:?}", error);
                    web_sys::console::error_1(&error_str.clone().into());
                    handle_command_error(error_str, set_error_state);
                }
            }
        });
    }

    pub fn skip_phase(&self) {
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            match invoke_command_no_args(event_names::timer::SKIP_PHASE).await {
                Ok(result) => {
                    // skip_phase returns (Phase, Phase, TimerState) tuple
                    if let Ok((_, _, state)) =
                        serde_wasm_bindgen::from_value::<(Phase, Phase, TimerState)>(result)
                    {
                        set_timer_state.set(state);
                    } else {
                        web_sys::console::error_1(
                            &"Failed to parse timer state tuple from skip_phase".into()
                        );
                    }
                }
                Err(error) => {
                    let error_str = format!("Failed to skip phase: {:?}", error);
                    web_sys::console::error_1(&error_str.clone().into());
                    handle_command_error(error_str, set_error_state);
                }
            }
        });
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
        let remaining = state.remaining_seconds();
        let total = match &state {
            TimerState::Working { configuration, .. } => {
                configuration.get_phase_duration_seconds(Phase::Work)
            }
            TimerState::ShortBreak { configuration, .. } => {
                configuration.get_phase_duration_seconds(Phase::ShortBreak)
            }
            TimerState::LongBreak { configuration, .. } => {
                configuration.get_phase_duration_seconds(Phase::LongBreak)
            }
            TimerState::Paused { paused_from, .. } => {
                let phase = Self::get_current_phase_static(paused_from);
                paused_from
                    .configuration()
                    .get_phase_duration_seconds(phase)
            }
            TimerState::Idle { .. } => return 0.0,
        };

        if total == 0 {
            0.0
        } else {
            ((total - remaining) as f64 / total as f64) * 100.0
        }
    }

    pub fn get_session_display(&self) -> String {
        let state = self.timer_state.get();
        let session_count = state.session_count();
        let sessions_until_long_break =
            state.configuration().sessions_until_long_break as u32;
        format!(
            "Session {}/{}",
            session_count % sessions_until_long_break + 1,
            sessions_until_long_break
        )
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
        let state = self.timer_state.get();
        (state.session_count()
            % state.configuration().sessions_until_long_break as u32)
            as usize
    }

    pub fn get_today_pomodoros(&self) -> u32 {
        // This would typically come from a stats service, for now return session count
        let state = self.timer_state.get();
        state.session_count()
    }

    pub fn get_task_pomodoros(&self) -> u32 {
        // This would typically track pomodoros for the active task
        // For now, return a default value
        0
    }

    pub fn get_active_entity_id(&self) -> Option<String> {
        self.timer_state.get().active_entity_id().map(|x| x.to_string())
    }
}
