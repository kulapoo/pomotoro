use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use crate::store::{setup_timer_events, setup_phase_complete_events};
use pomotoro_domain::{TimerStateWithTask, TimerState, Phase, TimerStatus, events};

// Re-export domain types for timer components
pub use pomotoro_domain::{Phase, TimerStatus, TimerState};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub struct TimerPageState {
    pub timer_state: ReadSignal<TimerState>,
    pub timer_with_task: ReadSignal<TimerStateWithTask>,
    pub set_timer_state: WriteSignal<TimerState>,
}

impl TimerPageState {
    pub fn new() -> Self {
        let (timer_state, set_timer_state) = signal(TimerState::default());
        let (timer_with_task, set_timer_with_task) = signal(TimerStateWithTask::new(TimerState::default(), None));
        
        // Initial load of timer state
        Effect::new(move |_| {
            spawn_local(async move {
                let result = invoke(events::timer::GET_STATE_WITH_TASK, JsValue::NULL).await;
                if let Ok(state_with_task) = serde_wasm_bindgen::from_value::<TimerStateWithTask>(result) {
                    set_timer_with_task.set(state_with_task.clone());
                    set_timer_state.set(state_with_task.timer_state);
                }
            });
        });

        // Setup event listeners
        setup_timer_events(set_timer_state);
        setup_phase_complete_events();
        
        Self {
            timer_state,
            timer_with_task,
            set_timer_state,
        }
    }
}