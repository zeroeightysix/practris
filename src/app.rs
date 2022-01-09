use eframe::{egui, epi};
use gilrs::{EventType, GamepadId, Gilrs};

use crate::singleplayer::SingleplayerGame;
use crate::State;

const APP_NAME: &'static str = "zersis";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    settings_open: bool,
    game: SingleplayerGame,
    #[cfg_attr(feature = "persistence", serde(skip))]
    gilrs: Gilrs,
    gamepad: Option<GamepadId>,
}

impl App {
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
}

impl Default for App {
    fn default() -> Self {
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
            settings_open: false,
            game: SingleplayerGame::new(),
            gilrs,
            gamepad
        }
    }
}

impl epi::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        // continuous rendering
        ctx.request_repaint();

        self.update_gamepad();
        self.game.update(&ctx.input().keys_down, self.gamepad.map(|id| self.gilrs.gamepad(id)));

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(format!("{} {}", APP_NAME, VERSION));
                egui::warn_if_debug_build(ui);

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    if ui.selectable_label(self.settings_open, "⚙ Settings").clicked() {
                        self.settings_open = !self.settings_open;
                    }
                })
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("{:?}", self.game.state));
            self.game.render(ui);
        });

        if self.settings_open {
            egui::SidePanel::right("settings_panel")
                .min_width(150.0)
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Settings");
                        ui.separator()
                    })
                });
        }
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        env_logger::init();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, APP_NAME).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, APP_NAME, self);
    }

    fn name(&self) -> &str {
        APP_NAME
    }
}
