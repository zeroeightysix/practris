use std::collections::HashSet;
use eframe::egui::Key;

use gilrs::{Axis, Button, Gamepad};
use libtetris::*;
use serde::{Deserialize, Serialize};
use crate::game::Event;

pub trait InputSource {
    fn controller(&self, keys: &HashSet<Key>, gamepad: Option<Gamepad<'_>>) -> Controller;
    fn update(
        &mut self,
        board: &Board<ColoredRow>,
        events: &[Event],
        incoming: u32,
    );
}

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug)]
pub struct UserInput {
    keyboard: Config<Key>,
    gamepad: Config<GamepadControl>,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
struct Config<T> {
    left: T,
    right: T,
    rotate_left: T,
    rotate_right: T,
    hard_drop: T,
    soft_drop: T,
    hold: T,
}

impl Default for Config<Key> {
    fn default() -> Self {
        Config {
            left: Key::ArrowLeft,
            right: Key::ArrowRight,
            rotate_left: Key::Z,
            rotate_right: Key::X,
            hard_drop: Key::Space,
            soft_drop: Key::ArrowDown,
            hold: Key::C,
        }
    }
}

impl Default for Config<GamepadControl> {
    fn default() -> Self {
        Config {
            left: GamepadControl::Button(Button::DPadLeft),
            right: GamepadControl::Button(Button::DPadRight),
            rotate_left: GamepadControl::Button(Button::South),
            rotate_right: GamepadControl::Button(Button::East),
            hard_drop: GamepadControl::Button(Button::DPadUp),
            soft_drop: GamepadControl::Button(Button::DPadDown),
            hold: GamepadControl::Button(Button::LeftTrigger),
        }
    }
}

impl InputSource for UserInput {
    fn controller(&self, keys: &HashSet<Key>, gamepad: Option<Gamepad<'_>>) -> Controller {
        Controller {
            left: self.read_input(keys, gamepad, self.keyboard.left, self.gamepad.left),
            right: self.read_input(keys, gamepad, self.keyboard.right, self.gamepad.right),
            rotate_left: self.read_input(
                keys,
                gamepad,
                self.keyboard.rotate_left,
                self.gamepad.rotate_left,
            ),
            rotate_right: self.read_input(
                keys,
                gamepad,
                self.keyboard.rotate_right,
                self.gamepad.rotate_right,
            ),
            hard_drop: self.read_input(
                keys,
                gamepad,
                self.keyboard.hard_drop,
                self.gamepad.hard_drop,
            ),
            soft_drop: self.read_input(
                keys,
                gamepad,
                self.keyboard.soft_drop,
                self.gamepad.soft_drop,
            ),
            hold: self.read_input(keys, gamepad, self.keyboard.hold, self.gamepad.hold),
        }
    }

    fn update(&mut self, _: &Board<ColoredRow>, _: &[Event], _: u32) {

    }
}

impl UserInput {
    fn read_input(
        &self,
        keys: &HashSet<Key>,
        controller: Option<Gamepad<'_>>,
        keyboard: Key,
        gamepad: GamepadControl,
    ) -> bool {
        keys.contains(&keyboard)
            || controller.map_or(false, |c| match gamepad {
            GamepadControl::Button(button) => c.is_pressed(button),
            GamepadControl::PositiveAxis(axis) => c.value(axis) > 0.5,
            GamepadControl::NegativeAxis(axis) => c.value(axis) < -0.5,
        })
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
enum GamepadControl {
    Button(Button),
    NegativeAxis(Axis),
    PositiveAxis(Axis),
}
