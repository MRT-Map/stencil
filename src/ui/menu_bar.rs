use egui::scroll_area::ScrollBarVisibility;
use egui_notify::ToastLevel;
use tracing::info;

use crate::{
    App,
    info_windows::{
        changelog::ChangelogPopup, info::InfoPopup, licenses::LicensesPopup, manual::ManualPopup,
        quit::QuitPopup,
    },
    project::{component_editor::ComponentEditorWindow, project_editor::ProjectEditorWindow},
    settings::SettingsWindow,
    shortcut::{ShortcutAction, UiButtonWithShortcutExt},
    ui::notif::NotifLogWindow,
};

impl App {
    pub fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                macro_rules! button {
                    ($ui:ident, fn $label:literal, $f:block) => {
                        if $ui.button($label).clicked() {
                            info!(label = $label, "Clicked menu item");
                            $f
                        }
                    };
                    ($ui:ident, fn $label:literal, $f:block, $action:expr) => {
                        if $ui.button_with_shortcut($label, $action, &mut self.shortcut_settings).clicked() {
                            info!(label = $label, "Clicked menu item");
                            $f
                        }
                    };
                    ($ui:ident, window $label:literal, $window:expr) => {
                        if $ui.button($label).clicked() {
                            info!(label = $label, "Clicked menu item");
                            self.open_dock_window($window)
                        }
                    };
                    ($ui:ident, window $label:literal, $window:expr, $action:expr) => {
                        if $ui.button_with_shortcut($label, $action, &mut self.shortcut_settings).clicked() {
                            info!(label = $label, "Clicked menu item");
                            self.open_dock_window($window)
                        }
                    };
                }

                ui.menu_button(format!("Stencil v{}", env!("CARGO_PKG_VERSION")), |ui| {
                    button!(ui, fn "Info", {
                        self.add_popup(InfoPopup);
                    });
                    button!(ui, fn "Changelog", {
                        self.add_popup(ChangelogPopup);
                    });
                    button!(ui, fn "Manual", {
                        self.add_popup(ManualPopup);
                    });
                    button!(ui, fn "Licenses", {
                        self.add_popup(LicensesPopup::default());
                    });
                    ui.separator();
                    button!(ui, window "Settings", SettingsWindow::default(), ShortcutAction::SettingsWindow);
                    ui.separator();
                    button!(ui, fn "Quit", {
                        self.add_popup(QuitPopup);
                    }, ShortcutAction::Quit);
                });
                ui.menu_button("File", |ui| {
                    // button!(ui, commands, "Open...", ProjectEv::Open);
                    // button!(ui, commands, "Reload", ProjectEv::Reload);
                    button!(ui, fn "Save", {
                        self.project.save_notif(&mut self.ui.notifs);
                    }, ShortcutAction::SaveProject);
                });
                ui.menu_button("Edit", |ui| {
                    button!(ui, fn "Undo", {
                        self.undo(ui.ctx());
                    }, ShortcutAction::Undo);
                    button!(ui, fn "Redo", {
                        self.undo(ui.ctx());
                    }, ShortcutAction::Redo);
                });
                ui.menu_button("View", |ui| {
                    // button!(ui, commands, "Component List", OpenComponentListEv);
                    button!(ui, window "Component Editor", ComponentEditorWindow, ShortcutAction::ComponentEditorWindow);
                    button!(ui, window "Project Editor", ProjectEditorWindow::default(), ShortcutAction::ProjectEditorWindow);
                    // button!(ui, commands, "History", OpenHistoryViewerEv);
                    button!(ui, window "Notification Log", NotifLogWindow, ShortcutAction::NotifLogWindow);
                    ui.separator();
                    button!(ui, fn "Reset Layout", {
                        self.ui.dock_layout.reset();
                        self.reset_map_window();
                    });
                });
                #[cfg(debug_assertions)]
                {
                    ui.menu_button("Debug", |ui| {
                        if ui.button("Trigger Warning").clicked() {
                            info!(label = "Trigger Warning", "Clicked menu item");
                            self.ui
                                .notifs
                                .push("Warning Triggered", ToastLevel::Warning);
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
