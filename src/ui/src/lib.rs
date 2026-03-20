pub mod components;
pub mod hooks;
pub mod views;

use wasm_bindgen::prelude::*;

use components::layout::App;

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
