use std::collections::HashSet;

use gilrs::{EventType, GamepadId, Gilrs};
use nannou::prelude::*;
use nannou_egui::{egui, Egui};

use crate::singleplayer::SingleplayerGame;
use crate::State;

const APP_NAME: &'static str = "zersis";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct Model {
    egui: Egui,
    ui: Ui,
    keys_pressed: HashSet<Key>,
    game: SingleplayerGame,
    gilrs: Gilrs,
    gamepad: Option<GamepadId>,
    ui_occupation: (f32, f32),
}

struct Ui {
    settings_open: bool,
}

impl Model {
    pub fn update(&mut self, update: Update) {
        self.update_gamepad();
        self.game.update(&self.keys_pressed, self.gamepad.map(|id| self.gilrs.gamepad(id)));

        let egui = &mut self.egui;
        egui.set_elapsed_time(update.since_start);
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
                    })
                }).response.rect.width()
        } else {
            0.
        };

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
        draw.background().color(CORNFLOWERBLUE);

        let (header, sidebar) = model.ui_occupation;
        let window_rect = app.window_rect();

        let draw_space = Rect::from_corners(
            window_rect.top_left() - Point2::new(0., header),
            window_rect.bottom_right() - Point2::new(sidebar, 0.),
        );

        model.game.render(&draw, draw_space);

        draw.to_frame(app, &frame).unwrap();

        model.egui.draw_to_frame(&frame).unwrap();
    }

    pub fn from_app(app: &App) -> Self {
        let window_id = app.new_window()
            .key_pressed(Model::key_pressed)
            .key_released(Model::key_released)
            .raw_event(Model::raw_event)
            .view(Model::view)
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

        Self {
            egui,
            ui: Ui {
                settings_open: false
            },
            keys_pressed: HashSet::new(),
            game: SingleplayerGame::new(),
            gilrs,
            gamepad,
            ui_occupation: (0.0, 0.0),
        }
    }
}