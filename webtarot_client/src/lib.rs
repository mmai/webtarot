#![recursion_limit = "2048"]

#[macro_use]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

mod api;
mod app;
mod components;
mod utils;
mod views;
mod sound_player;

use wasm_bindgen::prelude::*;

pub(crate) use webtarot_protocol as protocol;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_logger::init();
    yew::start_app::<crate::app::App>();
    Ok(())
}
