#![feature(let_else)]
#![warn(clippy::all, rust_2018_idioms)]

use std::collections::HashSet;

use gilrs::Gamepad;
use nannou::prelude::*;

use crate::model::Model;

mod model;
mod game;
mod input;
mod game_ui;
mod singleplayer;

trait State {
    fn update(&mut self, keys: &HashSet<Key>, gamepad: Option<Gamepad<'_>>);
    fn render(&self, draw: &Draw, rect: Rect);
}

fn main() {
    env_logger::init();

    nannou::app(model)
        .update(update)
        .run();
}

fn update(_app: &nannou::App, model: &mut Model, update: Update) {
    model.update(update);
}

fn model(app: &nannou::App) -> Model {
    Model::from_app(app)
}
