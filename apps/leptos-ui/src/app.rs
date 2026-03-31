use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::app_vm::AppViewModel;
use crate::components::{ScreenBlockerProvider, Sidebar};
use crate::pages::{SettingsPage, TaskDirectoryPage, TaskFormPage, TimerPage};
use crate::utils::ViewModel;

#[component]
pub fn App() -> impl IntoView {
    // Create app-level ViewModel
    let app_vm = StoredValue::new(AppViewModel::new());

    // Provide to children via context
    provide_context(app_vm);

    view! {
        <ScreenBlockerProvider>
            <Router>
                <AppLayout />
            </Router>
        </ScreenBlockerProvider>
    }
}

#[component]
fn AppLayout() -> impl IntoView {
    view! {
        <>
            <Sidebar />

            <main class="ml-[250px] p-8 min-h-screen bg-slate-50 transition-all duration-200" id="mainContent">
                <Routes fallback=|| view! { <NotFound /> }>
                    // Root redirects to timer
                    <Route path=path!("/") view=|| view! { <TimerPage /> } />
                    <Route path=path!("/timer") view=|| view! { <TimerPage /> } />

                    // Task routes - all in one place
                    <Route path=path!("/tasks") view=|| view! { <TaskDirectoryPage /> } />
                    <Route path=path!("/tasks/new") view=|| view! { <TaskFormPage /> } />
                    <Route path=path!("/tasks/:id/edit") view=|| view! { <TaskFormPage /> } />

                    // Settings route
                    <Route path=path!("/settings") view=|| view! { <SettingsPage /> } />
                </Routes>
            </main>
        </>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-[50vh] text-center">
            <h1 class="text-6xl font-bold text-slate-800 mb-4">"404"</h1>
            <p class="text-xl text-slate-600">"Page not found"</p>
        </div>
    }
}
