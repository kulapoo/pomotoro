use leptos::prelude::*;
use crate::pages::timer::{TimerDisplay, TimerControls};
use crate::pages::timer::timer_state::TimerPageState;
use crate::components::PageHeader;

#[component]
pub fn TimerPage() -> impl IntoView {
    let page_state = TimerPageState::new();
    
    view! {
        <div class="w-full">
            <PageHeader 
                title="Timer".to_string()
            />
            <div class="bg-white rounded-2xl p-8 shadow-lg text-center">
                <TimerDisplay 
                    timer_state=page_state.timer_state
                    timer_with_task=page_state.timer_with_task
                />
                <TimerControls
                    timer_state=page_state.timer_state
                    set_timer_state=page_state.set_timer_state
                />
            </div>
        </div>
    }
}