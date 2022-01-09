#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

use std::collections::HashSet;

use eframe::egui::{Key, Ui};
#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};
use gilrs::Gamepad;

pub use app::App;

mod app;
mod game;
mod input;
mod game_ui;
mod singleplayer;

trait State {
    fn update(&mut self, keys: &HashSet<Key>, gamepad: Option<Gamepad<'_>>);
    fn render(&mut self, ui: &mut Ui);
}

// ----------------------------------------------------------------------------
// When compiling for web:

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let app = App::default();
    eframe::start_web(canvas_id, Box::new(app))
}
