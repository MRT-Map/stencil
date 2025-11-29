use std::sync::Arc;

use tracing::info;

use crate::{
    App,
    map::MapWindow,
    mode::EditorMode,
    project::{project_editor::ProjectEditorWindow, skin::SkinType},
    shortcut::{ShortcutAction, UiButtonWithShortcutExt},
};

impl MapWindow {
    pub fn toolbar(app: &mut App, ui: &mut egui::Ui) {
        let old_mode = app.mode;
        egui::TopBottomPanel::top("toolbar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                macro_rules! button {
                    ($text:literal, $next_state:expr) => {
                        ui.selectable_value(&mut app.mode, $next_state, $text)
                    };
                }

                button!("Select", EditorMode::Select);
                button!("Nodes", EditorMode::Nodes);

                ui.separator();
                ui.label("Create...");
                button!("Point", EditorMode::CreatePoint);
                button!("Line", EditorMode::CreateLine);
                button!("Area", EditorMode::CreateArea);

                let mut in_namespace = |app: &mut App| {
                    ui.label("in ns.");
                    if !app.project.new_component_ns.is_empty()
                        && app
                            .project
                            .namespaces
                            .get(&app.project.new_component_ns)
                            .is_none_or(|a| !*a)
                    {
                        app.project.new_component_ns.clear();
                    }

                    egui::ComboBox::from_id_salt("toolbar_namespace")
                        .selected_text(if app.project.new_component_ns.is_empty() {
                            egui::RichText::new("select...").italics()
                        } else {
                            (&app.project.new_component_ns).into()
                        })
                        .show_ui(ui, |ui| {
                            if app
                                .project
                                .namespaces
                                .iter()
                                .filter(|(_, vis)| **vis)
                                .map(|(ns, _)| {
                                    ui.selectable_value(
                                        &mut app.project.new_component_ns,
                                        ns.to_owned(),
                                        ns,
                                    );
                                })
                                .next()
                                .is_none()
                            {
                                ui.horizontal(|ui| {
                                    ui.label("Create or load namespaces in the");
                                    if ui.small_button("Project Editor").clicked() {
                                        app.open_dock_window(ProjectEditorWindow);
                                    }
                                });
                            }
                        });
                };
                if let Some(skin) = app.project.skin() {
                    match &app.mode {
                        EditorMode::CreatePoint => {
                            in_namespace(app);
                            ui.label("with type");
                            egui::ComboBox::from_id_salt("toolbar_type")
                                .selected_text(
                                    app.ui
                                        .map
                                        .created_point_type
                                        .as_ref()
                                        .unwrap_or_else(|| skin.get_type("simplePoint").unwrap())
                                        .widget_text(ui, &egui::TextStyle::Button),
                                )
                                .show_ui(ui, |ui| {
                                    for ty in &skin.types {
                                        if !matches!(ty.as_ref(), SkinType::Point { .. }) {
                                            continue;
                                        }
                                        ui.selectable_value(
                                            &mut app.ui.map.created_point_type,
                                            Some(Arc::clone(ty)),
                                            ty.widget_text(ui, &egui::TextStyle::Button),
                                        );
                                    }
                                });
                        }
                        EditorMode::CreateLine => {
                            in_namespace(app);
                            ui.label("with type");
                            egui::ComboBox::from_id_salt("toolbar_type")
                                .selected_text(
                                    app.ui
                                        .map
                                        .created_line_type
                                        .as_ref()
                                        .unwrap_or_else(|| skin.get_type("simpleLine").unwrap())
                                        .widget_text(ui, &egui::TextStyle::Button),
                                )
                                .show_ui(ui, |ui| {
                                    for ty in &skin.types {
                                        if !matches!(ty.as_ref(), SkinType::Line { .. }) {
                                            continue;
                                        }
                                        ui.selectable_value(
                                            &mut app.ui.map.created_line_type,
                                            Some(Arc::clone(ty)),
                                            ty.widget_text(ui, &egui::TextStyle::Button),
                                        );
                                    }
                                });
                        }
                        EditorMode::CreateArea => {
                            in_namespace(app);
                            ui.label("with type");
                            egui::ComboBox::from_id_salt("toolbar_type")
                                .selected_text(
                                    app.ui
                                        .map
                                        .created_area_type
                                        .as_ref()
                                        .unwrap_or_else(|| skin.get_type("simpleArea").unwrap())
                                        .widget_text(ui, &egui::TextStyle::Button),
                                )
                                .show_ui(ui, |ui| {
                                    for ty in &skin.types {
                                        if !matches!(ty.as_ref(), SkinType::Area { .. }) {
                                            continue;
                                        }
                                        ui.selectable_value(
                                            &mut app.ui.map.created_area_type,
                                            Some(Arc::clone(ty)),
                                            ty.widget_text(ui, &egui::TextStyle::Button),
                                        );
                                    }
                                });
                        }
                        _ => {}
                    }
                }

                ui.separator();

                if app.project.path.is_none() {
                    ui.label(
                        egui::RichText::new(" THIS IS A SCRATCHPAD - NOTHING WILL BE SAVED ")
                            .background_color(egui::Color32::LIGHT_RED)
                            .color(egui::Color32::BLACK),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if ui
                        .button_with_shortcut(
                            "Reset View",
                            ShortcutAction::ResetMapView,
                            &mut app.shortcut_settings,
                        )
                        .clicked()
                    {
                        app.map_reset_view();
                    }
                    if let Some(prev_cursor_world_pos) = app.ui.map.cursor_world_pos {
                        ui.label(format!(
                            "x: {} z: {} \u{1f50d}: {:.2}",
                            prev_cursor_world_pos.x.round() as i32,
                            prev_cursor_world_pos.y.round() as i32,
                            app.ui.map.zoom
                        ));
                    } else {
                        ui.label(format!("\u{1f50d}: {:.2}", app.ui.map.zoom));
                    }

                    ui.separator();
                });
            });
        });

        if old_mode != app.mode {
            app.ui.map.created_nodes.clear();
            info!(mode=?app.mode, "Mode changed");
            app.status_default(ui.ctx());
        }
    }
}
