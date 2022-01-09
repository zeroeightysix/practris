use eframe::{egui, epi};

const APP_NAME: &'static str = "zersis";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    settings_open: bool,
    // this how you opt-out of serialization of a member
    // #[cfg_attr(feature = "persistence", serde(skip))]
    // value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            settings_open: false
        }
    }
}

impl epi::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
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
            // render game
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
