use std::collections::HashSet;

use gilrs::Gamepad;
use nannou::prelude::*;
use rand::{Rng, SeedableRng, thread_rng};
use rand_pcg::Pcg64Mcg;

use crate::game::{Game, GameConfig, PlayerUpdate};
use crate::ui::SingleplayerGameUi;
use crate::input::InputSource;
use crate::wgpu::Texture;

const RESET_TIME: usize = 40;

pub struct SingleplayerGame {
    ui: SingleplayerGameUi,
    texture: Texture,
    game: Game,
    pub(crate) input: Box<dyn InputSource>,
    pub state: State,
    piece_rng: Pcg64Mcg,
    garbage_rng: Pcg64Mcg,
    reset_countdown: f32,
}

#[derive(Debug)]
pub enum State {
    Playing,
    GameOver(u32),
    Starting(u32),
}

impl SingleplayerGame {
    pub fn new(texture: Texture, input: Box<dyn InputSource>) -> Self {
        let mut thread_rng = thread_rng();
        let mut rng = Pcg64Mcg::from_seed(thread_rng.gen());
        let mut garbage_rng = Pcg64Mcg::from_seed(thread_rng.gen());
        let game = Game::new(GameConfig::fast_config(), &mut rng);

        Self {
            ui: SingleplayerGameUi::new(&game, "amogus".to_string(), texture.clone()),
            texture,
            game,
            input,
            state: State::Starting(300),
            piece_rng: rng,
            garbage_rng,
            reset_countdown: 1.,
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
            let events = self.game.update(controller, &mut self.piece_rng, &mut self.garbage_rng);
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

        let actions = self.input.actions(keys, gamepad);
        if actions.reset {
            match self.reset_countdown {
                x if x <= 0. => {
                    self.reset_countdown = 1.;

                    let seed = thread_rng().gen();
                    let mut rng = Pcg64Mcg::from_seed(seed);
                    let game = Game::new(GameConfig::fast_config(), &mut rng);
                    self.ui = SingleplayerGameUi::new(&game, "amogus".to_string(), self.texture.clone());
                    self.game = game;
                    self.state = State::Playing;
                    self.piece_rng = rng;
                },
                _ => self.reset_countdown -= 1. / RESET_TIME as f32,
            }
        } else {
            self.reset_countdown = 1.;
        }
    }

    fn render(&self, draw: &Draw, rect: Rect) {
        self.ui.draw(draw, rect);

        // To make the reset button feel slightly more 'tactile' - as in, not just a looping value,
        // we introduce a percentage (20%) of the time the reset bind is pressed, that no visual
        // indicator is shown.
        // This also means that after a reset happens, the bar doesn't immediately start filling in
        // again, which also improves the feel of this function
        const RESET_IDLE: f32 = 0.8;
        if self.reset_countdown < RESET_IDLE {
            let reset_rect = Rect::from_wh(Vec2::new(rect.w() * (1. - self.reset_countdown / RESET_IDLE), 10.));
            let reset_rect = reset_rect.bottom_left_of(rect);
            draw.a::<nannou::draw::primitive::Rect>(reset_rect.into())
                .color(RED);
        }
    }
}