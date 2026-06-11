#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod component_actions;
mod info_windows;
mod logging;
mod map;
mod mode;
mod project;
mod settings;
mod shortcut;
mod ui;
mod utils;

use std::time::Instant;

use etcetera::AppStrategy;
use tracing::info;
use utils::EXECUTOR;

use crate::{
    logging::init_logger,
    mode::EditorMode,
    project::Project,
    settings::AppSettings,
    ui::UiState,
    utils::{file::FOLDERS, load_save::LoadSave},
};

fn main() -> eframe::Result<()> {
    init_logger();
    info!("Logger initialised");

    std::thread::spawn(|| -> ! {
        loop {
            EXECUTOR.try_tick();
        }
    });

    let app = App::new();

    eframe::run_native(
        "Stencil3",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icons/icon.png"))
                    .unwrap(),
            ),
            persistence_path: Some(FOLDERS.in_data_dir("eframe.json")),
            ..app.settings.ui.get_native_options()
        },
        Box::new(|cc| Ok(Box::new(app.init_cc(cc)))),
    )
}

#[derive(Default)]
pub struct App {
    ui: UiState,
    settings: AppSettings,
    mode: EditorMode,
    project: Project,
}

impl App {
    #[tracing::instrument(skip_all, name = "app_new")]
    fn new() -> Self {
        let mut app = Self::load_state();
        app.map_reset_view();
        if app.settings.map.clear_cache_on_startup {
            app.project.basemap.clear_cache_path();
        }
        app.ack_panic();
        app
    }
    #[tracing::instrument(skip_all, name = "app_init_cc")]
    fn init_cc(self, cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        self.settings.ui.reload_fonts(&cc.egui_ctx);
        self
    }
    #[tracing::instrument(skip_all, name = "app_load_state")]
    fn load_state() -> Self {
        Self {
            settings: AppSettings::load_state(),
            ui: UiState::load_state(),
            ..Self::default()
        }
    }
    #[tracing::instrument(skip_all, name = "app_save_state")]
    fn save_state(&self) {
        self.ui.dock_layout.save();
        self.settings.save_state();
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.project.load_skin(ctx);
        self.status_init();
        self.ui.notifs.process_notifs(&self.settings.misc);
        self.autosave(ctx);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let start = Instant::now();

        self.menu_bar(ui);
        self.dock(ui);
        self.popups(ui);
        self.notifs(ui);

        self.shortcuts(ui);

        let end = Instant::now();
        self.ui
            .mspf
            .add(ui.input(|a| a.time), (end - start).as_millis() as f32);
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
        self.save_state();
    }
}
