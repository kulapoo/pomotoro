#![allow(dead_code)]

mod app;
mod components;
mod store;
mod pages;
mod shared;

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
