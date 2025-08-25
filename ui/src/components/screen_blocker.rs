use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn ScreenBlocker(
    is_blocking: ReadSignal<bool>,
    blocking_message: ReadSignal<String>,
    on_disable: leptos::callback::Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || is_blocking.get()>
            <div class="fixed top-0 left-0 w-screen h-screen bg-black/95 z-[9999] flex items-center justify-center backdrop-blur-[10px]">
                <div class="text-center text-gray-50 max-w-2xl p-10 md:p-10 p-5">
                    <div class="text-5xl md:text-5xl text-3xl font-bold mb-6 gradient-text">
                        {move || blocking_message.get()}
                    </div>

                    <div class="mb-8">
                        <p class="text-xl md:text-xl text-lg mb-4 text-gray-300">"You're in a focused work session. Stay concentrated!"</p>
                        <p class="text-base md:text-base text-sm text-gray-400 italic">"Press ESC or click below if you need to temporarily disable blocking."</p>
                    </div>

                    <button
                        class="bg-red-500/20 text-red-300 border border-red-500/30 rounded-xl px-6 py-3 text-base font-medium cursor-pointer transition-all duration-200 hover:bg-red-500/30 hover:border-red-500/50 hover:text-red-200 hover:-translate-y-0.5"
                        on:click=move |_| on_disable.run(())
                    >
                        "Temporarily Disable Blocking"
                    </button>
                </div>
            </div>
        </Show>
    }
}

#[component]
#[allow(dead_code)]
pub fn ScreenBlockerProvider(children: Children) -> impl IntoView {
    let (is_blocking, set_is_blocking) = signal(false);
    let (blocking_message, set_blocking_message) =
        signal("Focus Time".to_string());
    let (is_disabled, set_is_disabled) = signal(false);

    Effect::new(move |_| {
        let handle_keydown = leptos::callback::Callback::new(
            move |event: web_sys::KeyboardEvent| {
                if event.key() == "Escape" && is_blocking.get() {
                    set_is_disabled.set(true);
                    set_is_blocking.set(false);
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
            })
        />
    }
}
