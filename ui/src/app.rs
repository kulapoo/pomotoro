use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::Sidebar;
use crate::pages::{SettingsPage, TaskDirectoryPage, TaskFormPage, TimerPage};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <AppLayout />
        </Router>
    }
}

#[component]
fn AppLayout() -> impl IntoView {
    view! {
        <>
            <Sidebar />

            <main class="main-content" id="mainContent">
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
        <div class="not-found">
            <h1>"404"</h1>
            <p>"Page not found"</p>
        </div>
    }
}
