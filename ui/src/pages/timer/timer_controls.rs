use leptos::prelude::*;
use crate::pages::timer::TimerViewModel;

#[component]
pub fn TimerControls(vm: StoredValue<TimerViewModel>) -> impl IntoView {
    
    let start_pause_action = move |_| {
        vm.with_value(|v| v.start_pause_timer());
    };
    
    let reset_action = move |_| {
        vm.with_value(|v| v.reset_timer());
    };
    
    let skip_action = move |_| {
        vm.with_value(|v| v.skip_phase());
    };
    
    view! {
        <>
            <button
                class="btn btn-primary"
                on:click=start_pause_action
            >
                {move || vm.with_value(|v| v.get_start_pause_button_text())}
            </button>

            <button
                class="btn btn-secondary"
                on:click=reset_action
            >
                "Reset"
            </button>

            <button
                class="btn btn-secondary"
                on:click=skip_action
            >
                "SKIP"
            </button>
        </>
    }
}