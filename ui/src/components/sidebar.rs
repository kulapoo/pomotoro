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
    let (collapsed, set_collapsed) =
        if let (Some(collapsed), Some(set_collapsed)) =
            (is_collapsed, set_is_collapsed)
        {
            (collapsed, set_collapsed)
        } else {
            let (local_collapsed, local_set_collapsed) = signal(false);
            (local_collapsed, local_set_collapsed)
        };

    let toggle_sidebar = move |_| {
        set_collapsed.update(|collapsed| *collapsed = !*collapsed);
    };

    view! {
        <nav class={move || format!("sidebar {}", if collapsed.get() { "collapsed" } else { "" })} id="sidebar">
            <div class="sidebar-header">
                <span class="sidebar-title">"Pomotoro"</span>
                <button class="toggle-btn" on:click=toggle_sidebar>"☰"</button>
            </div>
            <ul class="nav-menu">
                <li
                    class={move || format!(
                        "nav-item {}",
                        if current_section.get() == NavigationSection::Timer { "active" } else { "" }
                    )}
                    on:click=move |_| set_current_section.set(NavigationSection::Timer)
                >
                    <span class="nav-icon">"⏱️"</span>
                    <span class="nav-text">"Timer"</span>
                </li>
                <li
                    class={move || format!(
                        "nav-item {}",
                        if current_section.get() == NavigationSection::Tasks { "active" } else { "" }
                    )}
                    on:click=move |_| set_current_section.set(NavigationSection::Tasks)
                >
                    <span class="nav-icon">"📝"</span>
                    <span class="nav-text">"Tasks"</span>
                </li>
                <li
                    class={move || format!(
                        "nav-item {}",
                        if current_section.get() == NavigationSection::Settings { "active" } else { "" }
                    )}
                    on:click=move |_| set_current_section.set(NavigationSection::Settings)
                >
                    <span class="nav-icon">"⚙️"</span>
                    <span class="nav-text">"Settings"</span>
                </li>
            </ul>
        </nav>
    }
}
