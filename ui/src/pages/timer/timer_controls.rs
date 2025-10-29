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

    let complete_task_action = move |_| {
        vm.with_value(|v| v.complete_task());
    };

    let reset_task_action = move |_| {
        vm.with_value(|v| v.reset_task());
    };

    let can_toggle_start_pause = move || vm.with_value(|v| v.can_toggle_start_pause());

    let can_skip = move || vm.with_value(|v| v.can_skip());

    let skip_action = move |_| {
        vm.with_value(|v| v.skip_phase());
    };


    let is_task_completed = move || vm.with_value(|v| v.is_task_completed());

    view! {
        <>
            <button
                class="px-6 py-3 bg-indigo-600 text-white font-semibold rounded-md shadow-sm hover:bg-indigo-700 hover:shadow-md transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:shadow-sm disabled:hover:bg-indigo-600"
                on:click=start_pause_action
                disabled= move || !can_toggle_start_pause() || is_task_completed()
            >
                {move || vm.with_value(|v| v.get_start_pause_button_text())}
            </button>
            <button
                class="px-6 py-3 bg-slate-600 text-white font-semibold rounded-md shadow-sm hover:bg-slate-700 hover:shadow-md transition-all duration-200"
                on:click=reset_action
            >
                "Reset Timer"
            </button>
            <button
                class="px-6 py-3 bg-slate-600 text-white font-semibold rounded-md shadow-sm hover:bg-slate-700 hover:shadow-md transition-all duration-200"
                on:click=reset_task_action
            >
                "Reset Task"
            </button>
            <button
                class="px-6 py-3 bg-slate-600 text-white font-semibold rounded-md shadow-sm hover:bg-slate-700 hover:shadow-md transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:shadow-sm disabled:hover:bg-slate-600"
                on:click=skip_action
                disabled=move || !can_skip()
            >
                "Skip"
            </button>
            <button
                class="px-6 py-3 bg-slate-600 text-white font-semibold rounded-md shadow-sm hover:bg-slate-700 hover:shadow-md transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:shadow-sm disabled:hover:bg-slate-600"
                on:click=complete_task_action
                disabled=move || !is_task_completed()
            >
                "Complete Task"
            </button>
        </>
    }
}
