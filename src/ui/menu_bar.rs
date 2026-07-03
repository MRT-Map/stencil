use egui::scroll_area::ScrollBarVisibility;
use tracing::info;

use crate::{
    App,
    info_windows::{
        changelog::ChangelogPopup, info::InfoPopup, licenses::LicensesPopup, manual::ManualPopup,
        memorial::MemorialPopup, quit::QuitPopup,
    },
    map::tile_coord::TILE_CACHE,
    project::load_save::Pla2Format,
    shortcut::{ShortcutAction, UiButtonWithShortcutExt},
};

impl App {
    pub fn menu_button_fn(
        &mut self,
        location: &str,
        ui: &mut egui::Ui,
        label: &str,
        action: Option<ShortcutAction>,
    ) -> bool {
        let button = if let Some(action) = action {
            ui.button_with_shortcut(label, action, &mut self.settings.shortcut)
        } else {
            ui.button(label)
        };
        if button.clicked() {
            info!(label, "Clicked {location} item");
            return true;
        }
        false
    }
}

impl App {
    #[tracing::instrument(skip_all)]
    #[expect(clippy::too_many_lines)]
    pub fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                macro_rules! button {
                    ($ui:ident, $label:literal, $action:expr) => {
                        if self.menu_button_fn("menu bar", $ui, $label, Some($action)) {
                            self.run_action($ui, $action, None)
                        }
                    };
                    ($ui:ident, $label:literal, $action:expr, $f:block) => {
                        if self.menu_button_fn("menu bar", $ui, $label, $action) {
                            $f
                        }
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
                    button!(ui, "Settings", ShortcutAction::SettingsWindow);
                    ui.separator();
                    if ui.input(|i| i.modifiers.alt) {
                        button!(ui, "Stencil v1/v2 Memorial", None, {
                            self.add_popup(MemorialPopup);
                        });
                        ui.separator();
                    }
                    button!(ui, "Quit", Some(ShortcutAction::Escape), {
                        self.add_popup(QuitPopup);
                    });
                });
                ui.menu_button("File", |ui| {
                    button!(ui, "Open", ShortcutAction::OpenProject);
                    ui.menu_button("Import Namespaces", |ui| {
                        button!(ui, "Import pla3.zip", None, {
                            self.import_namespace_pla3_zip();
                        });
                        button!(ui, "Import pla2.json/msgpack", None, {
                            self.import_namespace_pla2();
                        });
                    });
                    ui.separator();
                    button!(ui, "Reload", ShortcutAction::ReloadProject);
                    ui.separator();
                    button!(ui, "Save", ShortcutAction::SaveProject);
                    button!(ui, "Save As", ShortcutAction::SaveProjectAs);
                    ui.menu_button("Export Namespaces", |ui| {
                        button!(ui, "Export pla3.zip", None, {
                            self.export_namespaces_pla3_zip();
                        });
                        button!(ui, "Export pla2.msgpack", None, {
                            self.export_namespaces_pla2(Pla2Format::MessagePack);
                        });
                        button!(ui, "Export pla2.json", None, {
                            self.export_namespaces_pla2(Pla2Format::Json);
                        });
                    });
                });
                ui.menu_button("Edit", |ui| {
                    button!(ui, "Undo", ShortcutAction::Undo);
                    button!(ui, "Redo", ShortcutAction::Redo);
                    ui.separator();
                    button!(ui, "Select All", ShortcutAction::SelectAll);
                    ui.separator();
                    button!(ui, "Copy", ShortcutAction::Copy);
                    button!(ui, "Cut", ShortcutAction::Cut);
                    button!(ui, "Delete", ShortcutAction::Delete);
                    ui.separator();
                    button!(ui, "Paste", ShortcutAction::Paste);
                    ui.separator();
                    button!(ui, "Clear Clipboard", None, {
                        self.map_clear_clipboard();
                    });
                });
                ui.menu_button("View", |ui| {
                    button!(ui, "Zoom In", ShortcutAction::ZoomMapIn);
                    button!(ui, "Zoom Out", ShortcutAction::ZoomMapOut);
                    ui.separator();
                    ui.label("Windows");
                    button!(ui, "Component", ShortcutAction::ComponentEditorWindow);
                    button!(ui, "Project", ShortcutAction::ProjectEditorWindow);
                    button!(ui, "History", ShortcutAction::HistoryViewerWindow);
                    button!(ui, "Notification Log", ShortcutAction::NotifLogWindow);
                    ui.separator();
                    button!(ui, "Reset Map View", ShortcutAction::ResetMapView);
                    button!(ui, "Clear Map Cache", None, {
                        self.project.basemap.clear_cache_path();
                        TILE_CACHE.lock().clear();
                    });
                    button!(ui, "Reset Window Layout", None, {
                        self.ui.dock_layout.reset();
                        self.map_reset_view();
                    });
                });
                #[cfg(debug_assertions)]
                {
                    ui.menu_button("Debug", |ui| {
                        if ui.button("Trigger Warning").clicked() {
                            info!(label = "Trigger Warning", "Clicked menu item");
                            crate::notif!(warning "Warning Triggered");
                        }
                        if ui.button("Trigger Panic").clicked() {
                            info!(label = "Trigger Panic", "Clicked menu item");
                            panic!("Panic Triggered");
                        }
                    });
                }
                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.label(format!(
                        "ms/frame: {:.3}",
                        self.ui.mspf.average().unwrap_or_default()
                    ));
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
