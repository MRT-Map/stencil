mod component_actions;
mod coord_conversion;
mod file;
mod info_windows;
mod load_save;
mod logging;
mod map;
mod mode;
mod project;
mod settings;
mod shortcut;
mod ui;

use std::{sync::LazyLock, time::Instant};

use async_executor::StaticExecutor;
use eframe::egui;
use lazy_regex::{Regex, lazy_regex};
use tracing::info;

use crate::{
    file::DATA_DIR,
    load_save::LoadSave,
    logging::init_logger,
    map::settings::MapSettings,
    mode::EditorMode,
    project::Project,
    settings::misc_settings::MiscSettings,
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
    map_settings: MapSettings,

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
        app
    }
    fn load_state() -> Self {
        let mut notifs = NotifState::default();
        let misc_settings = MiscSettings::load(&mut notifs);
        misc_settings.update_notif_duration();
        Self {
            ui: UiState {
                dock_layout: DockLayout::load(&mut notifs),
                ..UiState::default()
            },
            shortcut_settings: ShortcutSettings::load(&mut notifs),
            map_settings: MapSettings::load(&mut notifs),
            misc_settings,
            ..Self::default()
        }
    }
    fn save_state(&mut self) {
        self.ui.dock_layout.save(&mut self.ui.notifs);
        self.misc_settings.save(&mut self.ui.notifs);
        self.shortcut_settings.save(&mut self.ui.notifs);
        self.map_settings.save(&mut self.ui.notifs);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let start = Instant::now();
        self.project.load_skin(ctx);
        self.status_init(ctx);

        self.menu_bar(ctx);
        self.dock(ctx);
        self.popups(ctx);
        self.notifs(ctx);

        self.shortcuts(ctx);

        let end = Instant::now();
        self.ui
            .mspf
            .add(ctx.input(|a| a.time), (end - start).as_millis() as f32);
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
        self.save_state();
    }
}
