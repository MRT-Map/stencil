use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    App,
    component_actions::event::ComponentEv,
    coord_conversion::CoordConversionExt,
    project::{Project, SkinStatus, event::ProjectEv, pla3::PlaNode},
    settings::settings_ui_field,
    shortcut::{ShortcutAction, UiButtonWithShortcutExt},
    ui::dock::DockWindow,
};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
pub struct ProjectEditorWindow;

impl DockWindow for ProjectEditorWindow {
    fn title(self) -> String {
        "Project".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            macro_rules! button {
                ($ui:ident, $label:literal, $action:expr, $f:block) => {
                    if app.menu_button_fn("project editor menu", $ui, $label, $action) {
                        $f
                    }
                };
            }
            button!(ui, "Open", Some(ShortcutAction::OpenProject), {
                // commands.trigger(ProjectEv::Open);
            });
            button!(ui, "Reload", Some(ShortcutAction::ReloadProject), {
                // commands.trigger(ProjectEv::Reload);
            });
            button!(ui, "Save", Some(ShortcutAction::SaveProject), {
                app.project.save_notif(&mut app.ui.notifs);
            });
        });
        ui.separator();

        ui.label(format!(
            "Project directory: {}",
            app.project
                .path
                .as_ref()
                .map_or_else(|| "SCRATCHPAD".into(), |a| a.to_string_lossy())
        ));

        let id = "new namespace".into();
        let mut new_namespace = ui.data_mut(|d| d.get_persisted::<String>(id).unwrap_or_default());

        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .column(egui_extras::Column::auto().at_least(0.05))
            .column(egui_extras::Column::remainder())
            .column(egui_extras::Column::auto().at_least(0.05))
            .column(egui_extras::Column::auto().at_least(0.05))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("👁").on_hover_text("Visibility");
                });
                header.col(|ui| {
                    ui.label("Namespace");
                });
                header.col(|ui| {
                    ui.label("#");
                });
                header.col(|ui| {
                    ui.label(" ");
                });
            })
            .body(|mut body| {
                for (ns, mut vis) in app.project.namespaces.clone() {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            if app.project.path.is_none() {
                                return;
                            }
                            if ui.checkbox(&mut vis, "").changed() {
                                if vis {
                                    app.run_event(ProjectEv::Load(ns.clone()), ui.ctx());
                                } else {
                                    app.run_event(ProjectEv::Hide(ns.clone()), ui.ctx());
                                }
                            }
                        });
                        row.col(|ui| {
                            ui.collapsing(egui::RichText::new(&ns).code(), |ui| {
                                Self::component_list(app, ui, &ns);
                            });
                        });
                        let num_components = app.project.namespace_component_count(&ns);
                        row.col(|ui| {
                            ui.label(
                                num_components
                                    .as_ref()
                                    .map_or_else(|_| "-".into(), |a| format!("{a}")),
                            );
                        });
                        row.col(|ui| {
                            if ui
                                .add_enabled(
                                    matches!(num_components, Ok(0)),
                                    egui::Button::new("❌").fill(egui::Color32::DARK_RED),
                                )
                                .clicked()
                            {
                                app.run_event(ProjectEv::Delete(ns), ui.ctx());
                            }
                        });
                    });
                }

                body.row(20.0, |mut row| {
                    row.col(|_| ());
                    row.col(|ui| {
                        egui::TextEdit::singleline(&mut new_namespace)
                            .hint_text("New namespace")
                            .show(ui);
                    });
                    row.col(|ui| {
                        if ui
                            .add_enabled(
                                !new_namespace.is_empty()
                                    && !app.project.namespaces.contains_key(&new_namespace),
                                egui::Button::new("➕"),
                            )
                            .clicked()
                        {
                            app.run_event(
                                ProjectEv::Create(std::mem::take(&mut new_namespace)),
                                ui.ctx(),
                            );
                        }
                    });
                });
            });

        ui.memory_mut(|m| {
            m.data.insert_persisted(id, new_namespace);
        });

        ui.separator();

        match &app.project.skin_status {
            SkinStatus::Unloaded => {
                ui.colored_label(egui::Color32::ORANGE, "Skin is unloaded");
            }
            SkinStatus::Loading(_) => {
                ui.colored_label(egui::Color32::YELLOW, "Skin is loading");
            }
            SkinStatus::Failed(e) => {
                ui.colored_label(egui::Color32::RED, "Skin failed to load");
                ui.code(format!("{e:?}"));
            }
            SkinStatus::Loaded(_) => {
                ui.colored_label(egui::Color32::GREEN, "Skin is loaded");
            }
        }

        ui.separator();

        ui.heading("Configuration");
        ui.collapsing("Skin", |ui| {
            settings_ui_field(
                ui,
                &mut app.project.skin_url,
                Project::default().skin_url,
                Option::<&str>::None,
                |ui, value| {
                    ui.add(egui::TextEdit::singleline(value).desired_width(200.0));
                    ui.label("Skin URL");
                    if ui.button("Reload").clicked() {
                        app.project.skin_status = SkinStatus::Unloaded;
                    }
                },
            );
        });
        ui.collapsing("Basemap", |ui| {
            app.project.basemap.config_ui(ui);
        });
    }
}

impl ProjectEditorWindow {
    pub fn component_list(app: &mut App, ui: &mut egui::Ui, ns: &str) {
        let pos = ui.ctx().pointer_interact_pos();
        let is_clicked = ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary));
        let mut component_to_delete = None;
        let mut component_to_select = None;
        let mut is_hovering = false;
        egui_extras::TableBuilder::new(ui)
            .column(egui_extras::Column::remainder())
            .column(egui_extras::Column::auto())
            .columns(egui_extras::Column::auto(), 3)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("id");
                });
                header.col(|ui| {
                    ui.label("type");
                });
            })
            .body(|mut body| {
                for component in app
                    .project
                    .components
                    .iter()
                    .filter(|a| a.full_id.namespace == ns)
                    .sorted_by_key(|a| &a.full_id.id)
                {
                    body.row(20.0, |mut row| {
                        if app.ui.map.selected_components.contains(&component.full_id) {
                            row.set_selected(true);
                        }
                        row.col(|ui| {
                            ui.label(
                                egui::RichText::new(component.to_string())
                                    .code()
                                    .text_style(egui::TextStyle::Small),
                            );
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                            ui.label(component.ty.widget_text(ui, &egui::TextStyle::Body));
                        });
                        row.col(|ui| {
                            let Some(centre) = PlaNode::centre(component.nodes.iter().copied())
                            else {
                                return;
                            };
                            if ui.small_button("➡").clicked() {
                                app.ui.map.centre_coord = centre.to_geo_coord_f32();
                            }
                        });
                        row.col(|ui| {
                            if ui
                                .add(
                                    egui::Button::new("❌")
                                        .small()
                                        .fill(egui::Color32::DARK_RED),
                                )
                                .clicked()
                            {
                                component_to_delete = Some(component.to_owned());
                            }
                        });
                        if let Some(pos) = pos
                            && row.response().interact_rect.contains(pos)
                        {
                            is_hovering = true;
                            if is_clicked {
                                component_to_select = Some(component.full_id.clone());
                            }
                        }
                    });
                }
            });
        if let Some(component_to_delete) = component_to_delete {
            let component_to_delete = vec![component_to_delete];
            app.status_on_delete(&component_to_delete, ui.ctx());
            app.run_event(ComponentEv::Delete(component_to_delete), ui.ctx());
        }
        if let Some(component_to_select) = component_to_select {
            app.select_component(ui, component_to_select);
        }
        if is_hovering {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
}
