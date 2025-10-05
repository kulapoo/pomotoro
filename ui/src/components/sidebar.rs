use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[derive(Debug, Clone, PartialEq)]
pub enum NavigationSection {
    Timer,
    Tasks,
    Settings,
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let (collapsed, set_collapsed) = signal(false);
    let location = use_location();

    let toggle_sidebar = move |_| {
        set_collapsed.update(|collapsed| *collapsed = !*collapsed);
    };

    // Determine active section based on current path
    let is_active = move |path: &'static str| {
        let pathname = location.pathname.get();
        if path == "/timer" && (pathname == "/" || pathname == "/timer") {
            true
        } else {
            pathname.starts_with(path)
        }
    };

    view! {
        <nav class={move || format!("sidebar {}", if collapsed.get() { "collapsed" } else { "" })} id="sidebar">
            <div class="sidebar-header">
                <span class="sidebar-title">"Pomotoro"</span>
                <button class="toggle-btn" on:click=toggle_sidebar>"☰"</button>
            </div>
            <ul class="nav-menu">
                <li class={move || format!("nav-item {}", if is_active("/timer") { "active" } else { "" })}>
                    <A href="/timer" attr:class="nav-link">
                        <span class="nav-icon">"⏱️"</span>
                        <span class="nav-text">"Timer"</span>
                    </A>
                </li>
                <li class={move || format!("nav-item {}", if is_active("/tasks") { "active" } else { "" })}>
                    <A href="/tasks" attr:class="nav-link">
                        <span class="nav-icon">"📝"</span>
                        <span class="nav-text">"Tasks"</span>
                    </A>
                </li>
                <li class={move || format!("nav-item {}", if is_active("/settings") { "active" } else { "" })}>
                    <A href="/settings" attr:class="nav-link">
                        <span class="nav-icon">"⚙️"</span>
                        <span class="nav-text">"Settings"</span>
                    </A>
                </li>
            </ul>
        </nav>
    }
}
