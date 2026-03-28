mod component_actions;
mod coord_conversion;
mod file;
mod info_windows;
mod load_save;
mod logging;
mod map;
mod mode;
mod pointer;
mod project;
mod settings;
mod shortcut;
mod ui;

use std::{sync::LazyLock, time::Instant};

use async_executor::StaticExecutor;
use lazy_regex::{Regex, lazy_regex};
use tracing::info;

use crate::{
    file::DATA_DIR,
    load_save::LoadSave,
    logging::init_logger,
    map::settings::MapSettings,
    mode::EditorMode,
    project::Project,
    settings::{misc_settings::MiscSettings, window_settings::WindowSettings},
    shortcut::settings::ShortcutSettings,
    ui::{UiState, dock::DockLayout, notif::NotifState},
};

pub static EXECUTOR: StaticExecutor = StaticExecutor::new();
pub static URL_REPLACER: LazyLock<Regex> = lazy_regex!("[<>:/\\|?*\"]");

fn main() {
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
            persistence_path: Some(DATA_DIR.join("eframe.json")),
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
    map_settings: MapSettings,
    window_settings: WindowSettings,

    mode: EditorMode,
    project: Project,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut app = Self::load_state();
        app.map_reset_view();
        if app.map_settings.clear_cache_on_startup {
            app.project.basemap.clear_cache_path(&mut app.ui.notifs);
        }
        cc.egui_ctx.set_style_of(
            egui::Theme::Dark,
            app.window_settings.dark_mode_style.clone(),
        );
        cc.egui_ctx.set_style_of(
            egui::Theme::Light,
            app.window_settings.light_mode_style.clone(),
        );
        app
    }
    fn load_state() -> Self {
        let mut notifs = NotifState::default();
        Self {
            ui: UiState {
                dock_layout: DockLayout::load(&mut notifs),
                ..UiState::default()
            },
            shortcut_settings: ShortcutSettings::load(&mut notifs),
            map_settings: MapSettings::load(&mut notifs),
            window_settings: WindowSettings::load(&mut notifs),
            misc_settings: {
                let s = MiscSettings::load(&mut notifs);
                s.update_notif_duration();
                s
            },
            ..Self::default()
        }
    }
    fn save_state(&mut self) {
        self.ui.dock_layout.save(&mut self.ui.notifs);
        self.misc_settings.save(&mut self.ui.notifs);
        self.shortcut_settings.save(&mut self.ui.notifs);
        self.map_settings.save(&mut self.ui.notifs);
        self.window_settings.save(&mut self.ui.notifs);
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.project.load_skin(ctx);
        self.status_init(ctx);
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
