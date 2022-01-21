use std::collections::HashSet;

use gilrs::Gamepad;
use nannou::prelude::*;
use rand::{Rng, SeedableRng, thread_rng};
use rand_pcg::Pcg64Mcg;

use crate::game::{Game, GameConfig, PlayerUpdate};
use crate::ui::SingleplayerGameUi;
use crate::input::{InputSource, UserInput};

pub struct SingleplayerGame {
    ui: SingleplayerGameUi,
    game: Game,
    input: Box<dyn InputSource>,
    pub state: State,
    rng: Pcg64Mcg
}

#[derive(Debug)]
pub enum State {
    Playing,
    GameOver(u32),
    Starting(u32),
}

impl SingleplayerGame {
    pub fn new() -> Self {
        let seed = thread_rng().gen();
        let mut rng = Pcg64Mcg::from_seed(seed);

        let game = Game::new(GameConfig::fast_config(), &mut rng);

        Self {
            ui: SingleplayerGameUi::new(&game, "amogus".to_string()),
            game,
            input: Box::new(UserInput::default()),
            state: State::Starting(300),
            rng
        }
    }
}

impl crate::State for SingleplayerGame {
    fn update(&mut self, keys: &HashSet<Key>, gamepad: Option<Gamepad<'_>>) {
        let do_update = match self.state {
            State::GameOver(0) => {
                false
            }
            State::GameOver(ref mut delay) => {
                *delay -= 1;
                true
            }
            State::Starting(0) => {
                self.state = State::Playing;
                true
            }
            State::Starting(ref mut delay) => {
                *delay -= 1;
                false
            }
            State::Playing => true,
        };

        if do_update {
            let controller = self.input.controller(keys, gamepad);
            let events = self.game.update(controller, &mut self.rng.clone(), &mut self.rng);
            let update = PlayerUpdate {
                events,
                garbage_queue: 0
            };

            if let State::Playing = self.state {
                for event in &update.events {
                    use crate::game::Event::*;
                    match event {
                        GameOver => {
                            self.state = State::GameOver(300);
                        }
                        _ => {}
                    }
                }
            }

            self.ui.update(update);
        }
    }

    fn render(&self, draw: &Draw, rect: Rect) {
        self.ui.draw(draw, rect);
    }
}