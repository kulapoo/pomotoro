use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn TaskCycleControls(
    on_next: impl Fn() + Clone + 'static,
    on_previous: impl Fn() + Clone + 'static,
    position: ReadSignal<(usize, usize)>,
    is_active: ReadSignal<bool>,
) -> impl IntoView {
    Effect::new({
        let on_next = on_next.clone();
        let on_previous = on_previous.clone();
        move |_| {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    let on_next = on_next.clone();
                    let on_previous = on_previous.clone();
                    let listener = wasm_bindgen::closure::Closure::wrap(Box::new(
                        move |event: web_sys::KeyboardEvent| {
                            if event.ctrl_key() || event.meta_key() {
                                match event.key().as_str() {
                                    "Tab" if event.shift_key() => {
                                        event.prevent_default();
                                        on_previous();
                                    }
                                    "Tab" => {
                                        event.prevent_default();
                                        on_next();
                                    }
                                    _ => {}
                                }
                            }
                        },
                    ) as Box<dyn FnMut(_)>);

                    let _ = document.add_event_listener_with_callback(
                        "keydown",
                        listener.as_ref().unchecked_ref(),
                    );

                    listener.forget();
                }
            }
        }
    });

    view! {
        <div class="task-cycle-controls">
            <div class="cycle-position">
                {move || {
                    let (current, total) = position.get();
                    if total > 0 {
                        format!("Task {} of {} incomplete", current, total)
                    } else {
                        "No incomplete tasks".to_string()
                    }
                }}
            </div>
            <div class="cycle-buttons">
                <button
                    class="cycle-button previous"
                    on:click=move |_| on_previous()
                    disabled=move || !is_active.get() || position.get().1 <= 1
                    title="Previous incomplete task (Ctrl+Shift+Tab)"
                >
                    <svg
                        width="20"
                        height="20"
                        viewBox="0 0 20 20"
                        fill="none"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <path
                            d="M12 15L7 10L12 5"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                </button>
                <button
                    class="cycle-button next"
                    on:click=move |_| on_next()
                    disabled=move || !is_active.get() || position.get().1 <= 1
                    title="Next incomplete task (Ctrl+Tab)"
                >
                    <svg
                        width="20"
                        height="20"
                        viewBox="0 0 20 20"
                        fill="none"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <path
                            d="M8 5L13 10L8 15"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                </button>
            </div>
        </div>
    }
}