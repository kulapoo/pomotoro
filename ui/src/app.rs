use leptos::prelude::*;
use leptos::context::Provider;

use crate::pages::settings::ConfigResource;
use crate::pages::task::TaskResource;
use crate::pages::{TimerPage, TaskPage, SettingsPage};
use crate::components::{Sidebar, NavigationSection};

#[component]
pub fn App() -> impl IntoView {
    let config_resource = ConfigResource::new();
    let task_resource = TaskResource::new();
    
    // Navigation state management
    let (current_section, set_current_section) = signal(NavigationSection::Timer);

    // Render content based on current section
    let render_content = move || {
        match current_section.get() {
            NavigationSection::Timer => view! { <TimerPage /> }.into_any(),
            NavigationSection::Tasks => view! { <TaskPage /> }.into_any(),
            NavigationSection::Settings => view! { <SettingsPage /> }.into_any(),
        }
    };

    view! {
        <Provider value=config_resource>
            <Provider value=task_resource>
                <div class="min-h-screen" style="display: flex;">
                    // Sidebar navigation
                    <Sidebar 
                        current_section=current_section
                        set_current_section=set_current_section
                    />
                    
                    // Main content area
                    <main class="main-content">
                        <div class="content-area">
                            {render_content}
                        </div>
                    </main>
                </div>
            </Provider>
        </Provider>
    }
}
