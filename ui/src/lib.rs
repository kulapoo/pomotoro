#![allow(dead_code)] // Leptos component macros generate code that uses the fields

mod app;
mod components;
mod store;
mod pages;

use app::*;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
