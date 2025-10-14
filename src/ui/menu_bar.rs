use std::collections::VecDeque;

use egui::scroll_area::ScrollBarVisibility;
use tracing::info;

use crate::{
    App,
    event::{Event, Events},
    info_windows::InfoWindowEv,
};

impl App {
    pub fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let mut button = |ui: &mut egui::Ui, label: &str, event: Events| {
                    if ui.button(label).clicked() {
                        info!(label, "Clicked menu item");
                        self.events.push_back(event);
                    }
                };

                ui.menu_button(format!("Stencil v{}", env!("CARGO_PKG_VERSION")), |ui| {
                    button(ui, "Info", InfoWindowEv::Info.into());
                    button(ui, "Changelog", InfoWindowEv::Changelog.into());
                    button(ui, "Manual", InfoWindowEv::Manual.into());
                    button(ui, "Licenses", InfoWindowEv::Licenses.into());
                    ui.separator();
                    button(ui, "Quit", InfoWindowEv::Quit { confirm: false }.into());
                });
                // ui.menu_button("File", |ui| {
                //     button!(ui, commands, "Open...", ProjectEv::Open);
                //     button!(ui, commands, "Reload", ProjectEv::Reload);
                //     button!(ui, commands, "Save", ProjectEv::Save(false));
                // });
                // ui.menu_button("Edit", |ui| {
                //     button!(ui, commands, "Undo", HistoryEv::Undo);
                //     button!(ui, commands, "Redo", HistoryEv::Redo);
                // });
                // #[expect(clippy::cognitive_complexity)]
                // ui.menu_button("View", |ui| {
                //     button!(ui, commands, "Component List", OpenComponentListEv);
                //     button!(ui, commands, "Component Editor", OpenComponentEditorEv);
                //     button!(ui, commands, "Project", OpenProjectEditorEv);
                //     button!(ui, commands, "History", OpenHistoryViewerEv);
                //     button!(ui, commands, "Notification Log", OpenNotifLogViewerEv);
                //     ui.separator();
                //     button!(ui, commands, "Reset Layout", ResetPanelDockStateEv);
                // });
                // #[expect(clippy::cognitive_complexity)]
                // ui.menu_button("Settings", |ui| {
                //     button!(ui, commands, "Open All", OpenAllSettingsEv);
                //     ui.separator();
                //     button!(ui, commands, "Tilemap", TileSettingsEv::Open);
                //     button!(ui, commands, "Window", OpenWindowSettingsEv);
                //     button!(ui, commands, "Keymap", OpenKeymapSettingsEv);
                //     button!(ui, commands, "Misc", OpenMiscSettingsEv);
                // });
                // #[cfg(debug_assertions)]
                // {
                //     ui.menu_button("Debug", |ui| {
                //         if ui.button("Trigger Warning").clicked() {
                //             info!(label = "Trigger Warning", "Clicked menu item");
                //             NOTIF_LOG.push("Warning Triggered", ToastLevel::Warning);
                //         }
                //         if ui.button("Trigger Panic").clicked() {
                //             info!(label = "Trigger Panic", "Clicked menu item");
                //             panic!("Panic Triggered");
                //         }
                //     });
                // }
                ui.separator();

                egui::ScrollArea::horizontal()
                    .max_width(ui.available_width())
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.label(self.ui.status.clone());
                    });
            });
        });
    }
}
