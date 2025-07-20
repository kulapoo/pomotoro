mod app;
mod app_events;
mod components;
mod store;

use app::*;
use app_events::*;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
