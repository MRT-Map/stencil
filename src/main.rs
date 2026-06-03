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
use eyre::Result;
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

fn main() -> Result<()> {
    // std::panic::set_hook(Box::new(panic::panic));

    init_logger();
    info!("Logger initialised");

    std::thread::spawn(|| -> ! {
        loop {
            EXECUTOR.try_tick();
        }
    });

    eframe::run_native(
        "Stencil3",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icons/icon.png"))
                    .unwrap(),
            ),
            persistence_path: Some(FOLDERS.in_data_dir("eframe.json")),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )?;
    Ok(())
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
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut app = Self::load_state();
        app.map_reset_view();
        if app.settings.map.clear_cache_on_startup {
            app.project.basemap.clear_cache_path();
        }
        app
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
