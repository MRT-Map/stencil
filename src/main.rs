mod component_editor;
mod dirs_paths;
mod event;
mod info_windows;
mod logging;
mod map;
mod ui;

use std::collections::VecDeque;

use eframe::egui;
use eyre::Result;
use tracing::{error, info};

use crate::{
    event::{Event, Events},
    logging::init_logger,
    ui::{UiState, dock::DockLayout},
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
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap();
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct App {
    ui: UiState,

    #[serde(skip)]
    events: VecDeque<Events>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        //     Default::default()
        // }
        Self::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.menu_bar(ctx);
        self.dock(ctx);
        self.popups(ctx);

        while let Some(event) = self.events.pop_front() {
            event.log_react(self);
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
