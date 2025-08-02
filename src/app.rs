use leptos::prelude::*;
use leptos::context::Provider;
use leptos_router::*;

use crate::pages::settings::ConfigResource;
use crate::pages::task::TaskResource;
use crate::pages::{TimerPage, TaskPage, SettingsPage};
use crate::components::Navigation;

#[component]
pub fn App() -> impl IntoView {
    let config_resource = ConfigResource::new();
    let task_resource = TaskResource::new();

    view! {
        <Router>
            <Provider value=config_resource>
                <Provider value=task_resource>
                    <main class="min-h-screen pb-20 bg-gray-50">
                        <div class="container mx-auto px-4 py-6 max-w-lg">
                            <Routes fallback=|| "Page not found.">
                                <Route path="/" view=TimerPage/>
                                <Route path="/tasks" view=TaskPage/>
                                <Route path="/settings" view=SettingsPage/>
                            </Routes>
                        </div>
                        <Navigation />
                    </main>
                </Provider>
            </Provider>
        </Router>
    }
}
