use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use gilrs::{EventType, GamepadId, Gilrs};
use nannou::prelude::*;
use nannou::text::FontSize;
use nannou_egui::{egui, Egui};
use crate::config::UserSettings;
use crate::input::{Config, UserInput};

use crate::singleplayer::SingleplayerGame;
use crate::State;

const APP_NAME: &'static str = "practris";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct Model {
    egui: Egui,
    ui: Ui,
    keys_pressed: HashSet<Key>,
    game: SingleplayerGame,
    gilrs: Gilrs,
    gamepad: Option<GamepadId>,
    ui_occupation: (f32, f32),
    since_last_tick: Duration,
    settings: UserSettings,
}

struct Ui {
    settings_open: bool,
    keyboard: Option<(Config<Key>, Option<usize>)>,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            settings_open: false,
            keyboard: None,
        }
    }
}

impl Ui {
    pub fn is_paused(&self) -> bool {
        self.settings_open || self.keyboard.is_some()
    }
}

impl Model {
    /// Process a game tick, which processes inputs once, decreases timers, etc.
    /// In a correctly timed environment, this is done exactly 60 times per second.
    /// Some games may choose to rely on the FPS to be consistently 60, and thus tick when the game is rendered.
    /// practris allows itself more than 60 fps, thus we separate this logic and only tick when necessary.
    fn tick(&mut self) {
        self.update_gamepad();
        self.game.update(&self.keys_pressed, self.gamepad.map(|id| self.gilrs.gamepad(id)));
    }

    /// Render the game and process a tick if applicable.
    ///
    /// Ticks may not happen when the game is rendered above 60 fps, where some frames will be rendered without a game tick being processed.
    pub fn update(&mut self, update: Update) {
        if !self.ui.is_paused() {
            // how many ticks should have passed since the last tick?
            const TICK_STRIDE: f32 = 1000. / 60.;
            let diff = (update.since_start - self.since_last_tick).as_millis();
            let pass = diff as f32 / TICK_STRIDE; // cast is okay: overflow only if you left practris open for longer than you or your children will live
            if pass >= 60. * 10. {
                // the game hasn't seen a tick update in over 10s: let's not care and skip time.
                log::info!("Skipping ticks as the game is lagging behind for >10s");
                self.since_last_tick = update.since_start;
            } else if pass >= 1. {
                let to_process = pass as usize;
                self.since_last_tick = self.since_last_tick + Duration::from_millis((to_process as f32 * TICK_STRIDE) as u64);
                for _ in 0..to_process {
                    self.tick();
                }
            }
        } else {
            self.since_last_tick = update.since_start;
        }

        self.egui.set_elapsed_time(update.since_start);
        self.show_ui();
    }

    fn show_ui(&mut self) {
        let egui = &mut self.egui;
        let frame_ctx = egui.begin_frame();
        let ctx = &frame_ctx.context();

        let header_height = egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(format!("{} {}", APP_NAME, VERSION));
                egui::warn_if_debug_build(ui);

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    if ui.selectable_label(self.ui.settings_open, "⚙ Settings").clicked() {
                        self.ui.settings_open = !self.ui.settings_open;
                    }
                })
            });
        }).response.rect.max.y;

        let sidebar_width = if self.ui.settings_open {
            egui::SidePanel::right("settings_panel")
                .min_width(150.0)
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Settings");
                        ui.separator()
                    });

                    ui.add_enabled_ui(self.ui.keyboard.is_none(), |ui| {
                        if ui.button("Keyboard settings").clicked() {
                            let input = self.settings.input.clone();
                            self.ui.keyboard = Some((input.keyboard, None));
                        }
                    });
                }).response.rect.width()
        } else {
            0.
        };

        if let Some((keyboard, wait_for)) = &mut self.ui.keyboard {
            let mut open = true;
            egui::Window::new("Keyboard layout")
                .open(&mut open)
                .show(ctx, |ui| {
                    egui::Grid::new("kb_grid").show(ui, |ui| {
                        for (idx, (name, field)) in [
                            ("left", &mut keyboard.left),
                            ("right", &mut keyboard.right),
                            ("rotate_left", &mut keyboard.rotate_left),
                            ("rotate_right", &mut keyboard.rotate_right),
                            ("rotate_180", &mut keyboard.rotate_180),
                            ("hard_drop", &mut keyboard.hard_drop),
                            ("soft_drop", &mut keyboard.soft_drop),
                            ("hold", &mut keyboard.hold),
                        ].into_iter().enumerate() {
                            ui.label(name);
                            // if this key is currently waiting for input,
                            let label = if wait_for.map(|wf| wf == idx).unwrap_or(false) {
                                // also check for inputs
                                if let Some(key) = self.keys_pressed.iter().next() {
                                    *wait_for = None;
                                    *field = *key;
                                }

                                "Press a key".to_string()
                            } else {
                                format!("{field:?}")
                            };

                            if ui.button(label).clicked() {
                                *wait_for = Some(idx);
                            };
                            ui.end_row();
                        }
                    });
                    ui.label("Close to apply");
                });

            if !open {
                let (kb, _) = self.ui.keyboard.take().unwrap();
                self.settings.input = UserInput {
                    keyboard: kb,
                    ..Default::default()
                };
                self.game.input = Box::new(self.settings.input.clone());
            }
        }

        self.ui_occupation = (header_height, sidebar_width);
    }

    fn update_gamepad(&mut self) {
        while let Some(event) = self.gilrs.next_event() {
            match event.event {
                EventType::Connected => {
                    if self.gamepad.is_none() {
                        self.gamepad = Some(event.id);
                    }
                }
                EventType::Disconnected => {
                    if self.gamepad == Some(event.id) {
                        self.gamepad = None;
                    }
                }
                _ => {}
            }
        }
    }

    fn key_pressed(_app: &App, model: &mut Self, key: Key) {
        model.keys_pressed.insert(key);
    }

    fn key_released(_app: &App, model: &mut Self, key: Key) {
        model.keys_pressed.remove(&key);
    }

    fn raw_event(_app: &App, model: &mut Self, event: &nannou::winit::event::WindowEvent<'_>) {
        model.egui.handle_raw_event(event);
    }

    fn view(app: &App, model: &Self, frame: Frame<'_>) {
        let draw = app.draw();
        draw.background().color(GREY);

        let (header, sidebar) = model.ui_occupation;
        let window_rect = app.window_rect();

        let draw_space = Rect::from_corners(
            window_rect.top_left() - Point2::new(0., header),
            window_rect.bottom_right() - Point2::new(sidebar, 0.),
        );

        model.game.render(&draw, draw_space);
        if model.ui.is_paused() {
            draw.rect()
                .xy(window_rect.xy())
                .wh(window_rect.wh())
                .color(Rgba::new(0., 0., 0., 0.7));
            draw.text("PAUSED")
                .font_size((draw_space.w() / 40.) as FontSize)
                .w(draw_space.w()) // for some reason, if the w is not specified, at large sizes the text will wrap (PAUS\nED)
                .xy(draw_space.xy())
                .center_justify();
        }
        draw.to_frame(app, &frame).unwrap();

        model.egui.draw_to_frame(&frame).unwrap();
    }

    fn closed(_: &App, model: &mut Self) {
        if let Some(dir) = get_config_file() {
            match ron::ser::to_string_pretty(&model.settings, Default::default()) {
                Ok(cfg) => {
                    if let Err(e) = std::fs::write(dir, cfg) {
                        log::error!("Failed to write configuration: {e}")
                    }
                }
                Err(e) => {
                    log::error!("Failed to serialize configuration: {e}");
                }
            };
        } else {
            log::error!("Could not get config dir in order to save configuration");
        }
    }

    pub fn from_app(app: &App) -> Self {
        let window_id = app.new_window()
            .key_pressed(Model::key_pressed)
            .key_released(Model::key_released)
            .raw_event(Model::raw_event)
            .view(Model::view)
            .closed(Model::closed)
            .build()
            .unwrap();

        let egui = Egui::from_window(&app.window(window_id).unwrap());

        let gilrs = Gilrs::new().unwrap_or_else(|e| match e {
            gilrs::Error::NotImplemented(g) => {
                log::info!("Gamepads are not supported on this platform.");
                g
            }
            e => {
                log::error!("Failure initializing gamepad support: {}", e);
                panic!()
            }
        });
        let gamepad = gilrs.gamepads().next().map(|(id, _)| id);

        let assets = app.assets_path().unwrap();
        let texture = wgpu::Texture::from_path(app, assets.join("skin.png")).unwrap();

        let settings = if let Some(dir) = get_config_file() {
            if let Ok(src) = std::fs::read_to_string(dir) {
                ron::from_str(src.as_str())
                    .unwrap_or_else(|e| {
                        log::error!("Failed to read config: {e}!");
                        UserSettings::default()
                    })
            } else {
                log::info!("Configuration file not present: probably a first launch.");
                UserSettings::default()
            }
        } else {
            log::error!("Could not get config dir in order to load configuration");
            UserSettings::default()
        };

        Self {
            egui,
            ui: Ui::default(),
            keys_pressed: HashSet::new(),
            game: SingleplayerGame::new(texture, Box::new(settings.input.clone())),
            gilrs,
            gamepad,
            ui_occupation: (0.0, 0.0),
            since_last_tick: Duration::from_secs(0),
            settings,
        }
    }
}

fn get_config_file() -> Option<PathBuf> {
    dirs::config_dir()
        .map(|d| d.join(format!("{APP_NAME}.ron")))
}