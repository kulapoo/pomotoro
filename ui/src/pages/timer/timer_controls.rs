use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use domain::{TimerState, TimerStatus};
use domain::events;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn TimerControls(
    timer_state: ReadSignal<TimerState>,
    set_timer_state: WriteSignal<TimerState>,
) -> impl IntoView {
    let start_pause_timer = move |_| {
        let current_state = timer_state.get_untracked();
        spawn_local(async move {
            let command = match current_state.status() {
                TimerStatus::Running => events::timer::PAUSE,
                _ => events::timer::START,
            };
            let result = invoke(command, JsValue::NULL).await;
            if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                set_timer_state.set(state);
            }
        });
    };

    let reset_timer = move |_| {
        spawn_local(async move {
            let result = invoke(events::timer::RESET, JsValue::NULL).await;
            if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                set_timer_state.set(state);
            }
        });
    };

    let skip_phase = move |_| {
        spawn_local(async move {
            let result = invoke(events::timer::SKIP_PHASE, JsValue::NULL).await;
            if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                set_timer_state.set(state);
            }
        });
    };

    view! {
        <div class="flex gap-4 justify-center my-8 flex-wrap">
            <button
                class="px-6 py-3 md:px-6 md:py-3 px-5 py-2.5 border-none rounded-xl text-base md:text-base text-sm font-semibold cursor-pointer transition-all duration-200 shadow-lg min-w-20 md:min-w-20 min-w-18 bg-gradient-to-br from-green-500 to-green-600 text-white hover:-translate-y-0.5 hover:shadow-[var(--shadow-hover)] active:translate-y-0"
                on:click=start_pause_timer
            >
                {move || match timer_state.get().status() {
                    TimerStatus::Running => "Pause",
                    _ => "Start"
                }}
            </button>

            <button
                class="px-6 py-3 md:px-6 md:py-3 px-5 py-2.5 border-none rounded-xl text-base md:text-base text-sm font-semibold cursor-pointer transition-all duration-200 shadow-lg min-w-20 md:min-w-20 min-w-18 bg-gray-100 dark:bg-slate-700 text-gray-700 dark:text-slate-200 border border-gray-200 dark:border-slate-600 hover:bg-gray-200 dark:hover:bg-slate-600 hover:-translate-y-0.5 active:translate-y-0"
                on:click=reset_timer
            >
                "Reset"
            </button>

            <button
                class="px-6 py-3 md:px-6 md:py-3 px-5 py-2.5 border-none rounded-xl text-base md:text-base text-sm font-semibold cursor-pointer transition-all duration-200 shadow-lg min-w-20 md:min-w-20 min-w-18 bg-gray-100 dark:bg-slate-700 text-gray-700 dark:text-slate-200 border border-gray-200 dark:border-slate-600 hover:bg-gray-200 dark:hover:bg-slate-600 hover:-translate-y-0.5 active:translate-y-0"
                on:click=skip_phase
            >
                "Skip"
            </button>
        </div>

        <div class="mt-5">
            <span class={move || format!("inline-block px-4 py-1.5 rounded-2xl text-sm font-semibold uppercase tracking-wider {}", match timer_state.get().status() {
                TimerStatus::Running => "bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300 border border-green-300 dark:border-green-700",
                TimerStatus::Paused => "bg-yellow-100 dark:bg-yellow-900 text-yellow-700 dark:text-yellow-300 border border-yellow-300 dark:border-yellow-700",
                TimerStatus::Stopped => "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 border border-gray-300 dark:border-gray-600"
            })}>
                {move || match timer_state.get().status() {
                    TimerStatus::Running => "Running",
                    TimerStatus::Paused => "Paused",
                    TimerStatus::Stopped => "Stopped"
                }}
            </span>
        </div>
    }
}