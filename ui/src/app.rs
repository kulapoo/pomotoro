use leptos::prelude::*;

use crate::pages::{TimerPage, TaskPage, SettingsPage};
use crate::components::{Sidebar, NavigationSection};

#[component]
pub fn App() -> impl IntoView {
    let (current_section, set_current_section) = signal(NavigationSection::Timer);

    let render_content = move || {
        match current_section.get() {
            NavigationSection::Timer => view! { <TimerPage /> }.into_any(),
            NavigationSection::Tasks => view! { <TaskPage /> }.into_any(),
            NavigationSection::Settings => view! { <SettingsPage /> }.into_any(),
        }
    };

    view! {
        <div class="min-h-screen" style="display: flex;">
            <Sidebar
                current_section=current_section
                set_current_section=set_current_section
            />

            <main class="main-content">
                <div class="content-area">
                    {render_content}
                </div>
            </main>
        </div>
    }
}
