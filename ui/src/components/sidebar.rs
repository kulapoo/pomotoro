use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum NavigationSection {
    Timer,
    Tasks,
    Settings,
}

#[component]
pub fn Sidebar(
    current_section: ReadSignal<NavigationSection>,
    set_current_section: WriteSignal<NavigationSection>,
    #[prop(optional)] is_collapsed: Option<ReadSignal<bool>>,
    #[prop(optional)] set_is_collapsed: Option<WriteSignal<bool>>,
) -> impl IntoView {
    // Use provided state or create local state
    let (collapsed, set_collapsed) = if let (Some(collapsed), Some(set_collapsed)) = (is_collapsed, set_is_collapsed) {
        (collapsed, set_collapsed)
    } else {
        let (local_collapsed, local_set_collapsed) = signal(false);
        (local_collapsed, local_set_collapsed)
    };
    
    let toggle_sidebar = move |_| {
        set_collapsed.update(|collapsed| *collapsed = !*collapsed);
    };

    view! {
        <div class={move || format!("sidebar {}", if collapsed.get() { "collapsed" } else { "" })}>
            <div class="sidebar-content">
                // App brand/logo section
                <div class="app-brand">
                    <div class="bull-icon">"🐂"</div>
                    <span class="app-brand-text">"Pomotoro"</span>
                </div>

                // Navigation items
                <nav class="nav-menu">
                    // Timer section
                    <div 
                        class={move || format!(
                            "nav-item {}", 
                            if current_section.get() == NavigationSection::Timer { "active" } else { "" }
                        )}
                        on:click=move |_| set_current_section.set(NavigationSection::Timer)
                    >
                        <div class="nav-item-icon">"⏱️"</div>
                        <span class="nav-item-text">"Timer"</span>
                    </div>

                    // Tasks section
                    <div 
                        class={move || format!(
                            "nav-item {}", 
                            if current_section.get() == NavigationSection::Tasks { "active" } else { "" }
                        )}
                        on:click=move |_| set_current_section.set(NavigationSection::Tasks)
                    >
                        <div class="nav-item-icon">"📋"</div>
                        <span class="nav-item-text">"Tasks Directory"</span>
                    </div>

                    // Settings section
                    <div 
                        class={move || format!(
                            "nav-item {}", 
                            if current_section.get() == NavigationSection::Settings { "active" } else { "" }
                        )}
                        on:click=move |_| set_current_section.set(NavigationSection::Settings)
                    >
                        <div class="nav-item-icon">"⚙️"</div>
                        <span class="nav-item-text">"Settings"</span>
                    </div>
                </nav>
            </div>

            // Toggle button
            <button 
                class="sidebar-toggle"
                on:click=toggle_sidebar
            >
                {if collapsed.get() { "→" } else { "←" }}
            </button>
        </div>
    }
}