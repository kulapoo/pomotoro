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
        <nav
            class={move || format!(
                "fixed left-0 top-0 h-screen bg-white/95 backdrop-blur-[10px] border-r border-slate-200 shadow-md transition-all duration-200 ease-in-out z-50 {}",
                if collapsed.get() { "w-[60px]" } else { "w-[250px]" }
            )}
            id="sidebar"
        >
            <div class="flex items-center justify-between p-md border-b border-slate-200 h-16">
                <span class={move || format!(
                    "text-xl font-bold text-indigo-600 transition-opacity duration-200 {}",
                    if collapsed.get() { "opacity-0 w-0 overflow-hidden" } else { "opacity-100" }
                )}>"Pomotoro"</span>
                <button
                    class="p-2 text-slate-600 hover:text-indigo-600 hover:bg-slate-100 rounded-md transition-all duration-200 flex-shrink-0"
                    on:click=toggle_sidebar
                >"☰"</button>
            </div>
            <ul class="flex flex-col py-md">
                <li class="mb-xs">
                    <A href="/timer" attr:class={move || format!(
                        "flex items-center px-md py-3 mx-2 rounded-md transition-all duration-200 no-underline {}",
                        if is_active("/timer") {
                            "bg-indigo-600/10 text-indigo-600 border-l-4 border-indigo-600"
                        } else {
                            "text-slate-700 hover:bg-slate-100 hover:text-indigo-600"
                        }
                    )}>
                        <span class="text-2xl w-8 flex-shrink-0">"⏱️"</span>
                        <span class={move || format!(
                            "ml-3 font-medium transition-opacity duration-200 whitespace-nowrap {}",
                            if collapsed.get() { "opacity-0 w-0 overflow-hidden" } else { "opacity-100" }
                        )}>"Timer"</span>
                    </A>
                </li>
                <li class="mb-xs">
                    <A href="/tasks" attr:class={move || format!(
                        "flex items-center px-md py-3 mx-2 rounded-md transition-all duration-200 no-underline {}",
                        if is_active("/tasks") {
                            "bg-indigo-600/10 text-indigo-600 border-l-4 border-indigo-600"
                        } else {
                            "text-slate-700 hover:bg-slate-100 hover:text-indigo-600"
                        }
                    )}>
                        <span class="text-2xl w-8 flex-shrink-0">"📝"</span>
                        <span class={move || format!(
                            "ml-3 font-medium transition-opacity duration-200 whitespace-nowrap {}",
                            if collapsed.get() { "opacity-0 w-0 overflow-hidden" } else { "opacity-100" }
                        )}>"Tasks"</span>
                    </A>
                </li>
                <li class="mb-xs">
                    <A href="/settings" attr:class={move || format!(
                        "flex items-center px-md py-3 mx-2 rounded-md transition-all duration-200 no-underline {}",
                        if is_active("/settings") {
                            "bg-indigo-600/10 text-indigo-600 border-l-4 border-indigo-600"
                        } else {
                            "text-slate-700 hover:bg-slate-100 hover:text-indigo-600"
                        }
                    )}>
                        <span class="text-2xl w-8 flex-shrink-0">"⚙️"</span>
                        <span class={move || format!(
                            "ml-3 font-medium transition-opacity duration-200 whitespace-nowrap {}",
                            if collapsed.get() { "opacity-0 w-0 overflow-hidden" } else { "opacity-100" }
                        )}>"Settings"</span>
                    </A>
                </li>
            </ul>
        </nav>
    }
}
