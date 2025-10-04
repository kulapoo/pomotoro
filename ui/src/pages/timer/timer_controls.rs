use crate::pages::timer::TimerViewModel;
use leptos::prelude::*;

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

    let can_toggle_start_pause = move || vm.with_value(|v| v.can_toggle_start_pause());

    let can_skip = move || vm.with_value(|v| v.can_skip());

    let can_reset = move || vm.with_value(|v| v.can_reset());

    let is_task_completed = move || vm.with_value(|v| v.is_task_completed());

    view! {
        <>
            <button
                class="btn btn-primary"
                on:click=start_pause_action
                disabled= move || !can_toggle_start_pause() || is_task_completed()
            >
                {move || vm.with_value(|v| v.get_start_pause_button_text())}
            </button>

            <button
                class="btn btn-secondary"
                on:click=reset_action
                disabled=move || !can_reset() || is_task_completed()
            >
                "Reset"
            </button>

            <button
                class="btn btn-secondary"
                on:click=skip_action
                disabled=move || !can_skip() || is_task_completed()
            >
                "Skip"
            </button>
        </>
    }
}
