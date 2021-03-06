use std::collections::HashSet;

use gilrs::{Axis, Button, Gamepad};
use libtetris::*;
use nannou::event::Key;
use serde::{Deserialize, Serialize};
use crate::game::Event;

pub trait InputSource {
    fn controller(&self, keys: &HashSet<Key>, gamepad: Option<Gamepad<'_>>) -> Controller;
    fn actions(&self, keys: &HashSet<Key>, gamepad: Option<Gamepad<'_>>) -> GameAction;
    fn update(
        &mut self,
        board: &Board<ColoredRow>,
        events: &[Event],
        incoming: u32,
    );
}

pub struct GameAction {
    pub reset: bool
}

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug)]
pub struct UserInput {
    pub(crate) keyboard: Config<Key>,
    pub(crate) gamepad: Config<GamepadControl>,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Config<T> {
    pub(crate) left: T,
    pub(crate) right: T,
    pub(crate) rotate_left: T,
    pub(crate) rotate_right: T,
    pub(crate) rotate_180: T,
    pub(crate) hard_drop: T,
    pub(crate) soft_drop: T,
    pub(crate) hold: T,
    pub(crate) reset: T,
}

impl Default for Config<Key> {
    fn default() -> Self {
        Config {
            left: Key::Left,
            right: Key::Right,
            rotate_left: Key::Z,
            rotate_right: Key::X,
            rotate_180: Key::A,
            hard_drop: Key::Space,
            soft_drop: Key::Down,
            hold: Key::C,
            reset: Key::R,
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
            rotate_180: GamepadControl::Button(Button::North),
            hard_drop: GamepadControl::Button(Button::DPadUp),
            soft_drop: GamepadControl::Button(Button::DPadDown),
            hold: GamepadControl::Button(Button::LeftTrigger),
            reset: GamepadControl::Button(Button::West),
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
            rotate_180: self.read_input(
                keys,
                gamepad,
                self.keyboard.rotate_180,
                self.gamepad.rotate_180,
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

    fn actions(&self, keys: &HashSet<Key>, gamepad: Option<Gamepad<'_>>) -> GameAction {
        GameAction {
            reset: self.read_input(
                keys,
                gamepad,
                self.keyboard.reset,
                self.gamepad.reset,
            )
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
pub(crate) enum GamepadControl {
    Button(Button),
    NegativeAxis(Axis),
    PositiveAxis(Axis),
}
