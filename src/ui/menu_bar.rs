use egui::scroll_area::ScrollBarVisibility;
use egui_notify::ToastLevel;
use tracing::info;

use crate::{
    App,
    info_windows::InfoWindowEv,
    project::component_editor::ComponentEditorWindow,
    settings::SettingsWindow,
    shortcut::ShortcutAction,
    ui::{dock::ResetLayoutEv, notif::NotifLogWindow},
};

impl App {
    pub fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                macro_rules! button {
                    ($ui:ident, event $label:literal, $event:expr) => {
                        if $ui.button($label).clicked() {
                            info!(label = $label, "Clicked menu item");
                            self.events.push_back($event.into());
                        }
                    };
                    ($ui:ident, event $label:literal, $event:expr, $action:expr) => {
                        if $ui.add(egui::Button::new($label).shortcut_text($ui.ctx().format_shortcut(&self.shortcut_settings.action_to_keyboard($action)))).clicked() {
                            info!(label = $label, "Clicked menu item");
                            self.events.push_back($event.into());
                        }
                    };
                    ($ui:ident, window $label:literal, $window:expr) => {
                        if $ui.button($label).clicked() {
                            info!(label = $label, "Clicked menu item");
                            self.open_dock_window($window)
                        }
                    };
                    ($ui:ident, window $label:literal, $window:expr, $action:expr) => {
                        if $ui.add(egui::Button::new($label).shortcut_text($ui.ctx().format_shortcut(&self.shortcut_settings.action_to_keyboard($action)))).clicked() {
                            info!(label = $label, "Clicked menu item");
                            self.open_dock_window($window)
                        }
                    };
                }

                ui.menu_button(format!("Stencil v{}", env!("CARGO_PKG_VERSION")), |ui| {
                    button!(ui, event "Info", InfoWindowEv::Info);
                    button!(ui, event "Changelog", InfoWindowEv::Changelog);
                    button!(ui, event "Manual", InfoWindowEv::Manual);
                    button!(ui, event "Licenses", InfoWindowEv::Licenses);
                    ui.separator();
                    button!(ui, window "Settings", SettingsWindow::default(), ShortcutAction::SettingsWindow);
                    ui.separator();
                    button!(ui, event "Quit", InfoWindowEv::Quit { confirm: false }, ShortcutAction::Quit);
                });
                ui.menu_button("File", |_ui| {
                    // button!(ui, commands, "Open...", ProjectEv::Open);
                    // button!(ui, commands, "Reload", ProjectEv::Reload);
                    // button!(ui, commands, "Save", ProjectEv::Save(false));
                });
                ui.menu_button("Edit", |_ui| {
                    // button!(ui, commands, "Undo", HistoryEv::Undo);
                    // button!(ui, commands, "Redo", HistoryEv::Redo);
                });
                ui.menu_button("View", |ui| {
                    // button!(ui, commands, "Component List", OpenComponentListEv);
                    button!(ui, window "Component Editor", ComponentEditorWindow, ShortcutAction::ComponentEditorWindow);
                    // button!(ui, commands, "Project", OpenProjectEditorEv);
                    // button!(ui, commands, "History", OpenHistoryViewerEv);
                    button!(ui, window "Notification Log", NotifLogWindow, ShortcutAction::NotifLogWindow);
                    ui.separator();
                    button!(ui, event "Reset Layout", ResetLayoutEv);
                });
                #[cfg(debug_assertions)]
                {
                    ui.menu_button("Debug", |ui| {
                        if ui.button("Trigger Warning").clicked() {
                            info!(label = "Trigger Warning", "Clicked menu item");
                            self.ui
                                .notifs
                                .push("Warning Triggered", ToastLevel::Warning, &self.misc_settings);
                        }
                        if ui.button("Trigger Panic").clicked() {
                            info!(label = "Trigger Panic", "Clicked menu item");
                            panic!("Panic Triggered");
                        }
                    });
                }
                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.label(format!("ms/frame: {:.3}", self.ui.mspf.average().unwrap_or_default()));
                    ui.separator();

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        egui::ScrollArea::horizontal()
                            .max_width(ui.available_width())
                            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                            .show(ui, |ui| {
                                ui.label(self.ui.status.clone());
                            });
                    });
                });
            });
        });
    }
}
