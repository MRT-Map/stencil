mod component_editor;
mod dirs_paths;
mod event;
mod info_windows;
mod load_save;
mod logging;
mod map;
mod mode;
mod settings;
mod shortcut;
mod ui;

use std::collections::VecDeque;

use eframe::egui;
use eyre::Result;
use tracing::{error, info};

use crate::{
    dirs_paths::DATA_DIR,
    event::{Event, Events},
    load_save::LoadSave,
    logging::init_logger,
    mode::EditorMode,
    settings::misc_settings::MiscSettings,
    shortcut::settings::ShortcutSettings,
    ui::{UiState, dock::DockLayout, notif::NotifState},
};

fn main() {
    // std::panic::set_hook(Box::new(panic::panic));

    init_logger();
    info!("Logger initialised");

    eframe::run_native(
        "Stencil2",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../icons/icon.png")).unwrap(),
            ),
            persistence_path: Some(DATA_DIR.join("eframe_data")),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap();
}

#[derive(Default)]
struct App {
    ui: UiState,
    misc_settings: MiscSettings,
    shortcut_settings: ShortcutSettings,

    mode: EditorMode,

    events: VecDeque<Events>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self::load_state()
    }
    fn load_state() -> Self {
        let mut notifs = NotifState::default();
        let misc_settings = MiscSettings::load(&mut notifs, &MiscSettings::default());
        Self {
            ui: UiState {
                dock_layout: DockLayout::load(&mut notifs, &misc_settings),
                ..UiState::default()
            },
            shortcut_settings: ShortcutSettings::load(&mut notifs, &misc_settings),
            misc_settings,
            ..Self::default()
        }
    }
    fn save_state(&mut self) {
        self.ui
            .dock_layout
            .save(&mut self.ui.notifs, &self.misc_settings);
        self.misc_settings
            .save(&mut self.ui.notifs, &self.misc_settings);
        self.shortcut_settings
            .save(&mut self.ui.notifs, &self.misc_settings);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.menu_bar(ctx);
        self.dock(ctx);
        self.popups(ctx);
        self.notifs(ctx);

        self.shortcuts(ctx);

        while let Some(event) = self.events.pop_front() {
            event.log_react(ctx, self);
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
        self.save_state();
    }
}
