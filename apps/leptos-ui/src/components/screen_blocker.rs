use domain::event_names::{commands, ui_listeners};
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local as spawn_local_raw;
use web_sys::console;

use crate::utils::events::listen;
use crate::utils::invoke;

#[component]
pub fn ScreenBlocker(
    is_blocking: ReadSignal<bool>,
    blocking_message: ReadSignal<String>,
    on_disable: leptos::callback::Callback<()>,
) -> impl IntoView {
    move || {
        if !is_blocking.get() {
            return None;
        }

        let message = blocking_message.get();
        Some(view! {
            <div style="position:fixed;top:0;left:0;width:100vw;height:100vh;background:rgba(0,0,0,0.95);z-index:9999;display:flex;align-items:center;justify-content:center;backdrop-filter:blur(10px);">
                <div style="text-align:center;color:#f8fafc;max-width:672px;padding:40px;">
                    <div class="text-5xl font-bold mb-6 gradient-text" style="font-size:3rem;font-weight:700;margin-bottom:1.5rem;">
                        {message}
                    </div>
                    <div style="margin-bottom:2rem;">
                        <p style="font-size:1.25rem;margin-bottom:1rem;color:#cbd5e1;">"You're in a focused work session. Stay concentrated!"</p>
                        <p style="font-size:1rem;color:#94a3b8;font-style:italic;">"Press ESC or click below if you need to temporarily disable blocking."</p>
                    </div>
                    <button
                        style="background:rgba(239,68,68,0.2);color:#fca5a5;border:1px solid rgba(239,68,68,0.3);border-radius:12px;padding:12px 24px;font-size:1rem;font-weight:500;cursor:pointer;"
                        on:click=move |_| on_disable.run(())
                    >
                        "Temporarily Disable Blocking"
                    </button>
                </div>
            </div>
        })
    }
}

#[derive(Deserialize)]
struct ScreenBlockerActivatePayload {
    message: String,
}

#[component]
pub fn ScreenBlockerProvider(children: Children) -> impl IntoView {
    let (is_blocking, set_is_blocking) = signal(false);
    let (blocking_message, set_blocking_message) =
        signal("Focus Time".to_string());
    let (is_disabled, set_is_disabled) = signal(false);

    // Listen for screen_blocker:activate events from backend.
    // Use wasm_bindgen_futures::spawn_local directly to avoid any Leptos
    // owner/task-system overhead.
    spawn_local_raw(async move {
        console::log_1(&"[ScreenBlocker] registering listener".into());
        let callback = Closure::new(move |event: JsValue| {
            console::log_1(&"[ScreenBlocker] event received".into());
            console::log_1(&event);

            let payload = js_sys::Reflect::get(&event, &"payload".into())
                .unwrap_or(JsValue::NULL);

            console::log_1(&"[ScreenBlocker] payload:".into());
            console::log_1(&payload);

            match serde_wasm_bindgen::from_value::<ScreenBlockerActivatePayload>(
                payload,
            ) {
                Ok(data) => {
                    console::log_1(
                        &"[ScreenBlocker] parsed ok, showing blocker".into(),
                    );
                    set_blocking_message.set(data.message);
                    set_is_disabled.set(false);
                    set_is_blocking.set(true);

                    // Use wasm_bindgen_futures::spawn_local directly — this callback
                    // runs outside any Leptos reactive owner, so leptos::task::spawn_local
                    // should not be used here.
                    spawn_local_raw(async move {
                        let _ = invoke::<(), ()>(
                            commands::screen_blocker::ACTIVATE,
                            None,
                        )
                        .await;
                    });
                }
                Err(e) => {
                    console::log_1(
                        &format!(
                            "[ScreenBlocker] payload parse error: {:?}",
                            e
                        )
                        .into(),
                    );
                }
            }
        });

        listen(ui_listeners::screen_blocker::ACTIVATE, &callback).await;
        console::log_1(&"[ScreenBlocker] listener registered".into());
        callback.forget();
    });

    // Keydown handler for ESC
    Effect::new(move |_| {
        let handle_keydown = leptos::callback::Callback::new(
            move |event: web_sys::KeyboardEvent| {
                if event.key() == "Escape" && is_blocking.get() {
                    set_is_disabled.set(true);
                    set_is_blocking.set(false);

                    spawn_local_raw(async move {
                        let _ = invoke::<(), ()>(
                            commands::screen_blocker::DEACTIVATE,
                            None,
                        )
                        .await;
                    });
                }
            },
        );

        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
                    move |event: web_sys::Event| {
                        if let Ok(keyboard_event) =
                            event.dyn_into::<web_sys::KeyboardEvent>()
                        {
                            handle_keydown.run(keyboard_event);
                        }
                    },
                )
                    as Box<dyn FnMut(_)>);

                let _ = document.add_event_listener_with_callback(
                    "keydown",
                    closure.as_ref().unchecked_ref(),
                );
                closure.forget();
            }
        }
    });

    provide_context((is_blocking, set_is_blocking));
    provide_context((blocking_message, set_blocking_message));
    provide_context((is_disabled, set_is_disabled));

    view! {
        {children()}

        <ScreenBlocker
            is_blocking=is_blocking
            blocking_message=blocking_message
            on_disable=leptos::callback::Callback::new(move |_| {
                set_is_disabled.set(true);
                set_is_blocking.set(false);

                spawn_local_raw(async move {
                    let _ = invoke::<(), ()>(
                        commands::screen_blocker::DEACTIVATE,
                        None,
                    )
                    .await;
                });
            })
        />
    }
}
