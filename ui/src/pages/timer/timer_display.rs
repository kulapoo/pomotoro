use crate::components::circular_progress::CircularProgress;
use crate::pages::timer::TimerViewModel;
use crate::utils::ViewModel;
use leptos::prelude::*;

#[component]
#[allow(dead_code)]
pub fn TimerDisplay(vm: StoredValue<TimerViewModel>) -> impl IntoView {
    let timer_state = vm.with_value(|v| v.state());

    view! {
        <div class="mb-8">
            <h2 class="timer-label">
                {move || vm.with_value(|v| v.get_phase_name())}
            </h2>
        </div>

        <div class="relative flex items-center justify-center my-12">
            <CircularProgress
                progress=Signal::derive(move || vm.with_value(|v| v.get_progress_percentage()))
                phase=Signal::derive(move || timer_state.get().phase())
            />
            <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                <div class="text-center">
                    <div class="timer-display">
                        {move || vm.with_value(|v| v.format_time())}
                    </div>
                </div>
            </div>
        </div>
    }
}
