use egui::scroll_area::ScrollBarVisibility;
use egui_notify::ToastLevel;
use tracing::info;

use crate::{
    App,
    info_windows::{
        changelog::ChangelogPopup, info::InfoPopup, licenses::LicensesPopup, manual::ManualPopup,
        quit::QuitPopup,
    },
    project::{
        component_editor::ComponentEditorWindow, history_viewer::HistoryViewerWindow,
        project_editor::ProjectEditorWindow,
    },
    settings::SettingsWindow,
    shortcut::{ShortcutAction, UiButtonWithShortcutExt},
    ui::{dock::DockWindows, notif::NotifLogWindow},
};

impl App {
    pub fn menu_button_fn<'a>(
        &mut self,
        location: &str,
        ui: &mut egui::Ui,
        label: &str,
        action: Option<ShortcutAction>,
    ) -> bool {
        let button = if let Some(action) = action {
            ui.button_with_shortcut(label, action, &mut self.shortcut_settings)
        } else {
            ui.button(label)
        };
        if button.clicked() {
            info!(label, "Clicked {location} item");
            return true;
        }
        false
    }
    pub fn menu_button_window<'a>(
        &mut self,
        location: &str,
        ui: &mut egui::Ui,
        label: &str,
        action: Option<ShortcutAction>,
        window: impl Into<DockWindows>,
    ) {
        if self.menu_button_fn(location, ui, label, action) {
            self.open_dock_window(window);
        }
    }
}

impl App {
    pub fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                macro_rules! button {
                    ($ui:ident, $label:literal, $action:expr, $f:block) => {
                        if self.menu_button_fn("menu bar", $ui, $label, $action) {$f}
                    };
                    ($ui:ident, $label:literal, $action:expr, window $w:expr) => {
                        self.menu_button_window("menu bar", $ui, $label, $action, $w);
                    };
                }
                ui.menu_button(format!("Stencil v{}", env!("CARGO_PKG_VERSION")), |ui| {
                    button!(ui, "Info", None, {
                        self.add_popup(InfoPopup);
                    });
                    button!(ui, "Changelog", None, {
                        self.add_popup(ChangelogPopup);
                    });
                    button!(ui, "Manual", None, {
                        self.add_popup(ManualPopup);
                    });
                    button!(ui, "Licenses", None, {
                        self.add_popup(LicensesPopup::default());
                    });
                    ui.separator();
                    button!(ui, "Settings", Some(ShortcutAction::SettingsWindow), window SettingsWindow::default());
                    ui.separator();
                    button!(ui, "Quit", Some(ShortcutAction::Quit), {
                        self.add_popup(QuitPopup);
                    });
                });
                ui.menu_button("File", |ui| {
                    // button!(ui, commands, "Open...", ProjectEv::Open);
                    // button!(ui, commands, "Reload", ProjectEv::Reload);
                    button!(ui, "Save", Some(ShortcutAction::SaveProject), {
                        self.project.save_notif(&mut self.ui.notifs);
                    });
                });
                ui.menu_button("Edit", |ui| {
                    button!(ui, "Undo", Some(ShortcutAction::Undo), {
                        self.undo(ui.ctx());
                    });
                    button!(ui, "Redo", Some(ShortcutAction::Redo), {
                        self.redo(ui.ctx());
                    });
                    ui.separator();
                    button!(ui, "Copy", Some(ShortcutAction::Copy), {
                        self.copy_selected_components();
                    });
                    button!(ui, "Cut", Some(ShortcutAction::Cut), {
                        self.cut_selected_components(ctx);
                    });
                    button!(ui, "Delete", Some(ShortcutAction::Delete), {
                        self.delete_selected_components(ctx);
                    });
                    ui.separator();
                    button!(ui, "Paste", Some(ShortcutAction::Paste), {
                        self.paste_clipboard_components(ctx);
                    });
                });
                ui.menu_button("View", |ui| {
                    ui.label("Windows");
                    // button!(ui, commands, "Component List", OpenComponentListEv);
                    button!(ui, "Component", Some(ShortcutAction::ComponentEditorWindow), window ComponentEditorWindow);
                    button!(ui, "Project", Some(ShortcutAction::ProjectEditorWindow), window ProjectEditorWindow);
                    button!(ui, "History", Some(ShortcutAction::HistoryViewerWindow), window HistoryViewerWindow);
                    button!(ui, "Notification Log", Some(ShortcutAction::NotifLogWindow), window NotifLogWindow);
                    ui.separator();
                    button!(ui, "Reset Map View", Some(ShortcutAction::ResetMapView), {
                        self.reset_map_window();
                    });
                    button!(ui, "Reset Window Layout", None, {
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
