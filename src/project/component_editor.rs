use std::{collections::BTreeMap, sync::Arc};

use itertools::{Either, Itertools};
use serde::{Deserialize, Serialize};

use crate::{
    App,
    component_actions::event::ComponentEv,
    mode::EditorMode,
    project::{
        pla3::{PlaComponent, PlaNode},
        skin::SkinType,
    },
    ui::dock::DockWindow,
};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ComponentEditorWindow;

impl DockWindow for ComponentEditorWindow {
    fn title(self) -> String {
        "Component".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        let Some(skin) = app.project.skin() else {
            ui.heading("Waiting for skin...");
            return;
        };
        if [EditorMode::CreateLine, EditorMode::CreateArea].contains(&app.mode) {
            ui.heading(format!(
                "Creating {}",
                if app.mode == EditorMode::CreateLine {
                    "line"
                } else {
                    "area"
                }
            ));
            Self::show_position_data(ui, &app.ui.map.created_nodes);
            return;
        } else if app.mode == EditorMode::CreatePoint {
            ui.heading("Creating point");
            return;
        }
        let mut selected_components = app
            .ui
            .map
            .selected_components_mut(&mut app.project.components);
        if selected_components.is_empty() {
            if let Some(hovered_component) = &app.ui.map.hovered_component {
                ui.heading("Hovering over component:");
                ui.label(egui::RichText::new(hovered_component.to_string()).code());
            } else {
                ui.heading("Select components...");
            }
            ui.label(format!(
                "{} component(s) in clipboard",
                app.ui.map.clipboard.len()
            ));
            return;
        }

        let mut events_to_add = Vec::new();
        let mut old_selected_components = selected_components
            .iter()
            .map(|a| (**a).clone())
            .collect::<Vec<_>>();
        let mut add_event = |label: &str, selected_components: &[&mut PlaComponent]| {
            let new_components = selected_components
                .iter()
                .map(|a| (**a).clone())
                .collect::<Vec<_>>();
            events_to_add.push(ComponentEv::ChangeField {
                before: std::mem::take(&mut old_selected_components),
                after: new_components.clone(),
                label: label.into(),
            });
            old_selected_components = new_components;
        };

        ui.heading("Edit component data");
        ui.end_row();

        ui.horizontal(|ui| {
            let namespace = Itertools::exactly_one(
                selected_components
                    .iter()
                    .map(|c| &c.full_id.namespace)
                    .sorted()
                    .dedup(),
            )
            .cloned()
            .ok();
            egui::ComboBox::from_id_salt("component namespace")
                .selected_text(
                    namespace
                        .as_ref()
                        .map_or_else(|| egui::RichText::new("mixed").italics(), Into::into),
                )
                .width(25.0)
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    for (ns, vis) in &app.project.namespaces {
                        if !vis {
                            continue;
                        }
                        if ui
                            .selectable_label(namespace.as_ref() == Some(ns), ns)
                            .clicked()
                        {
                            for component in &mut selected_components {
                                component.full_id.namespace.clone_from(ns);
                            }
                            add_event("namespace", &selected_components);
                        }
                    }
                });

            if let Ok(component) = Itertools::exactly_one(selected_components.iter_mut()) {
                ui.code(&component.full_id.id);
            } else {
                ui.label(egui::RichText::new("mixed ids").italics());
            }
        });
        ui.end_row();

        let display_name = Itertools::exactly_one(
            selected_components
                .iter()
                .map(|c| &c.display_name)
                .sorted()
                .dedup(),
        )
        .ok();
        let mut new_display_name = display_name.cloned().unwrap_or_default();
        if ui
            .add(
                egui::TextEdit::singleline(&mut new_display_name)
                    .hint_text(if display_name.is_none() {
                        egui::RichText::new("mixed display names").italics()
                    } else {
                        "Display Name".into()
                    })
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            for component in &mut selected_components {
                component.display_name.clone_from(&new_display_name);
            }
            add_event("display_name", &selected_components);
        }
        ui.end_row();

        ui.separator();

        let skin_ty = Itertools::exactly_one(
            selected_components
                .iter()
                .map(|c| &c.ty)
                .sorted_by_key(|a| a.name())
                .dedup(),
        )
        .map(Arc::clone)
        .ok();
        let component_ty = Itertools::exactly_one(
            selected_components
                .iter()
                .map(|c| &c.ty)
                .map(|a| match &**a {
                    SkinType::Point { .. } => "point",
                    SkinType::Line { .. } => "line",
                    SkinType::Area { .. } => "area",
                })
                .dedup(),
        )
        .ok();
        egui::ComboBox::from_label("Component type")
            .selected_text(skin_ty.as_ref().map_or_else(
                || {
                    egui::RichText::new(if component_ty.is_some() {
                        "mixed skin types"
                    } else {
                        "mixed component types"
                    })
                    .italics()
                    .into()
                },
                |skin_ty| skin_ty.widget_text(ui, &egui::TextStyle::Button).into(),
            ))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                let Some(component_ty) = component_ty else {
                    return;
                };

                skin.types
                    .iter()
                    .filter(|ty| match component_ty {
                        "point" => matches!(&***ty, SkinType::Point { .. }),
                        "line" => matches!(&***ty, SkinType::Line { .. }),
                        "area" => matches!(&***ty, SkinType::Area { .. }),
                        _ => unreachable!(),
                    })
                    .sorted_by_key(|ty| ty.name())
                    .for_each(|ty| {
                        if ui
                            .selectable_label(
                                skin_ty.as_ref().is_some_and(|a| Arc::ptr_eq(a, ty)),
                                ty.widget_text(ui, &egui::TextStyle::Button),
                            )
                            .clicked()
                        {
                            for component in &mut selected_components {
                                component.ty = Arc::clone(ty);
                            }
                            add_event("ty", &selected_components);
                        }
                    });
            });
        ui.end_row();

        let layer = Itertools::exactly_one(
            selected_components
                .iter()
                .map(|c| c.layer)
                .sorted_by(f32::total_cmp)
                .dedup(),
        )
        .ok();
        let mut new_layer = layer.unwrap_or_default();
        if ui
            .add(
                egui::Slider::new(&mut new_layer, -10.0..=10.0).text(if layer.is_none() {
                    egui::RichText::new("Mixed Layers").italics()
                } else {
                    "Layer".into()
                }),
            )
            .changed()
        {
            for component in &mut selected_components {
                component.layer = new_layer;
            }
            add_event("layer", &selected_components);
        }

        ui.end_row();
        ui.separator();

        if component_ty == Some("line") {
            if ui.button("Reverse direction").clicked() {
                for component in &mut selected_components {
                    component.nodes = PlaNode::rev(component.nodes.iter().copied());
                }
                add_event("reverse", &selected_components);
            }
            ui.end_row();
            ui.separator();
        }

        let Ok(component) = Itertools::exactly_one(selected_components.iter_mut()) else {
            return;
        };

        ui.heading("Other Attributes");
        let changes = Self::table_editor(ui, Either::Left(&mut component.misc), "component ");
        if !changes.is_empty() {
            add_event(
                &changes
                    .iter()
                    .map(|a| {
                        a.strip_prefix("component ")
                            .and_then(|a| a.strip_suffix('/'))
                            .unwrap_or(a)
                    })
                    .join(" + "),
                &[component],
            );
        }

        ui.heading("Position data");
        Self::show_position_data(ui, &component.nodes);

        for ev in events_to_add {
            app.add_event(ev);
        }
    }
}

impl ComponentEditorWindow {
    fn field_editor(ui: &mut egui::Ui, v: &mut toml::Value, path: &str) -> Vec<String> {
        let mut changed = Vec::<String>::new();
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt(format!("{path} type"))
                .selected_text(match v {
                    toml::Value::String(_) => "String",
                    toml::Value::Integer(_) => "Integer",
                    toml::Value::Float(_) => "Float",
                    toml::Value::Boolean(_) => "Boolean",
                    toml::Value::Datetime(_) => "Datetime",
                    toml::Value::Array(_) => "Array",
                    toml::Value::Table(_) => "Table",
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(matches!(v, toml::Value::String(_)), "String")
                        .clicked()
                    {
                        *v = toml::Value::String(String::new());
                        changed.push(path.into());
                    }
                    if ui
                        .selectable_label(matches!(v, toml::Value::Integer(_)), "Integer")
                        .clicked()
                    {
                        *v = toml::Value::Integer(0);
                        changed.push(path.into());
                    }
                    if ui
                        .selectable_label(matches!(v, toml::Value::Float(_)), "Float")
                        .clicked()
                    {
                        *v = toml::Value::Float(0.0);
                        changed.push(path.into());
                    }
                    if ui
                        .selectable_label(matches!(v, toml::Value::Boolean(_)), "Boolean")
                        .clicked()
                    {
                        *v = toml::Value::Boolean(false);
                        changed.push(path.into());
                    }
                    if ui
                        .selectable_label(matches!(v, toml::Value::Datetime(_)), "Datetime")
                        .clicked()
                    {
                        *v = toml::Value::Datetime(toml::value::Datetime {
                            date: None,
                            time: None,
                            offset: None,
                        });
                        changed.push(path.into());
                    }
                    if ui
                        .selectable_label(matches!(v, toml::Value::Array(_)), "Array")
                        .clicked()
                    {
                        *v = toml::Value::Array(Vec::new());
                        changed.push(path.into());
                    }
                    if ui
                        .selectable_label(matches!(v, toml::Value::Table(_)), "Table")
                        .clicked()
                    {
                        *v = toml::Value::Table(toml::Table::new());
                        changed.push(path.into());
                    }
                });

            match v {
                toml::Value::String(v) => {
                    if ui.text_edit_multiline(v).changed() {
                        changed.push(path.into());
                    }
                }
                toml::Value::Integer(v) => {
                    if ui.add(egui::DragValue::new(v)).changed() {
                        changed.push(path.into());
                    }
                }
                toml::Value::Float(v) => {
                    if ui.add(egui::DragValue::new(v)).changed() {
                        changed.push(path.into());
                    }
                }
                toml::Value::Boolean(v) => {
                    if ui.checkbox(v, "").changed() {
                        changed.push(path.into());
                    }
                }
                toml::Value::Datetime(_) | toml::Value::Array(_) | toml::Value::Table(_) => {}
            }
        });

        match v {
            toml::Value::Array(v) => changed.extend(Self::array_editor(ui, v, &format!("{path}/"))),
            toml::Value::Table(v) => {
                changed.extend(Self::table_editor(
                    ui,
                    Either::Right(v),
                    &format!("{path}/"),
                ));
            }
            toml::Value::Datetime(v) => {
                let v_old = v.to_owned();
                let mut reset_date = false;
                let mut reset_time = false;
                let mut reset_offset = false;
                if let Some(date) = &mut v.date {
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::Button::new("❌").fill(egui::Color32::DARK_RED))
                            .clicked()
                        {
                            reset_date = true;
                        }
                        ui.add(
                            egui::DragValue::new(&mut date.year)
                                .custom_formatter(|a, _| format!("{a:04}")),
                        );
                        ui.label("-");
                        ui.add(
                            egui::DragValue::new(&mut date.month)
                                .custom_formatter(|a, _| format!("{a:02}"))
                                .range(1..=12),
                        );
                        ui.label("-");
                        ui.add(
                            egui::DragValue::new(&mut date.day)
                                .custom_formatter(|a, _| format!("{a:02}"))
                                .range(1..=31),
                        );
                    });
                } else if ui
                    .add(egui::Button::new("➕").right_text("Add date"))
                    .clicked()
                {
                    v.date = Some(toml::value::Date {
                        year: 1970,
                        month: 1,
                        day: 1,
                    });
                }

                if let Some(time) = &mut v.time {
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::Button::new("❌").fill(egui::Color32::DARK_RED))
                            .clicked()
                        {
                            reset_time = true;
                        }
                        ui.add(
                            egui::DragValue::new(&mut time.hour)
                                .custom_formatter(|a, _| format!("{a:02}"))
                                .range(0..=23),
                        );
                        ui.label(":");
                        ui.add(
                            egui::DragValue::new(&mut time.minute)
                                .custom_formatter(|a, _| format!("{a:02}"))
                                .range(0..=59),
                        );
                        ui.label(":");
                        ui.add(
                            egui::DragValue::new(&mut time.second)
                                .custom_formatter(|a, _| format!("{a:02}"))
                                .range(0..=59),
                        );
                        ui.label(".");
                        ui.add(
                            egui::DragValue::new(&mut time.nanosecond)
                                .custom_formatter(|a, _| format!("{a:09}"))
                                .range(0..=999_999_999),
                        );
                    });
                } else if ui
                    .add(egui::Button::new("➕").right_text("Add time"))
                    .clicked()
                {
                    v.time = Some(toml::value::Time {
                        hour: 0,
                        minute: 0,
                        second: 0,
                        nanosecond: 0,
                    });
                }

                if let Some(offset) = &mut v.offset {
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::Button::new("❌").fill(egui::Color32::DARK_RED))
                            .clicked()
                        {
                            reset_offset = true;
                        }

                        let (mut hours, mut minutes) = match offset {
                            toml::value::Offset::Z => (0, 0),
                            toml::value::Offset::Custom { minutes } => {
                                (*minutes / 60, *minutes % 60)
                            }
                        };
                        ui.add(
                            egui::DragValue::new(&mut hours)
                                .custom_formatter(|a, _| format!("{a:+03}")),
                        );
                        ui.label(":");
                        ui.add(
                            egui::DragValue::new(&mut minutes)
                                .custom_formatter(|a, _| format!("{a:02}"))
                                .range(0..=59),
                        );

                        let total_minutes = hours * 60 + minutes;
                        if total_minutes == 0 {
                            *offset = toml::value::Offset::Z;
                        } else {
                            *offset = toml::value::Offset::Custom {
                                minutes: total_minutes,
                            };
                        }
                    });
                } else if ui
                    .add(egui::Button::new("➕").right_text("Add offset"))
                    .clicked()
                {
                    v.offset = Some(toml::value::Offset::Z);
                }

                if reset_date {
                    v.date = None;
                }
                if reset_time {
                    v.time = None;
                }
                if reset_offset {
                    v.offset = None;
                }
                if v_old != *v {
                    changed.push(path.into());
                }
            }
            _ => {}
        }

        changed
    }

    fn array_editor(ui: &mut egui::Ui, array: &mut Vec<toml::Value>, path: &str) -> Vec<String> {
        let mut changed = Vec::<String>::new();
        let mut to_remove = None;
        let mut to_swap = None;

        let array_len = array.len();
        for (i, v) in array.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("❌").fill(egui::Color32::DARK_RED))
                    .clicked()
                {
                    to_remove = Some(i);
                }
                if ui.add_enabled(i != 0, egui::Button::new("⬆")).clicked() {
                    to_swap = Some((i, i - 1));
                }
                if ui
                    .add_enabled(i != array_len - 1, egui::Button::new("⬇"))
                    .clicked()
                {
                    to_swap = Some((i, i + 1));
                }
                changed.extend(Self::field_editor(ui, v, &format!("{path}{i}/")));
            });
        }
        if ui
            .add(egui::Button::new("➕").right_text("Add to array"))
            .clicked()
        {
            array.push(toml::Value::String(String::new()));
            changed.push(format!("{path}{array_len}"));
        }

        if let Some(to_remove) = to_remove {
            array.remove(to_remove);
            changed.push(format!("{path}{to_remove}"));
        }
        if let Some((a, b)) = to_swap {
            array.swap(a, b);
            changed.push(format!("{path}{a}&{b}"));
        }

        changed
    }

    fn table_editor(
        ui: &mut egui::Ui,
        mut table: Either<&mut BTreeMap<String, toml::Value>, &mut toml::Table>,
        path: &str,
    ) -> Vec<String> {
        let mut changed = Vec::<String>::new();

        let id = format!("{path} new key").into();
        let mut new_key = ui.data_mut(|d| d.get_persisted::<String>(id).unwrap_or_default());

        egui_extras::TableBuilder::new(ui)
            .id_salt(format!("{path} table"))
            .striped(true)
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::remainder())
            .body(|mut body| {
                let mut to_remove = None;

                let iter: Box<dyn Iterator<Item = (&String, &mut toml::Value)>> = match &mut table {
                    Either::Left(v) => Box::new(v.iter_mut()),
                    Either::Right(v) => Box::new(v.iter_mut()),
                };
                for (k, v) in iter {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if ui
                                    .add(egui::Button::new("❌").fill(egui::Color32::DARK_RED))
                                    .clicked()
                                {
                                    to_remove = Some(k.to_owned());
                                }
                                ui.label(k);
                            });
                        });
                        row.col(|ui| {
                            changed.extend(Self::field_editor(ui, v, &format!("{path}{k}")));
                        });
                    });
                }

                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut new_key).hint_text("Add to table"));
                    });
                    row.col(|ui| {
                        if ui
                            .add_enabled(
                                !new_key.is_empty()
                                    && match &table {
                                        Either::Left(v) => !v.contains_key(&new_key),
                                        Either::Right(v) => !v.contains_key(&new_key),
                                    },
                                egui::Button::new("➕"),
                            )
                            .clicked()
                        {
                            changed.push(format!("{path}{new_key}"));
                            match &mut table {
                                Either::Left(v) => {
                                    v.insert(
                                        std::mem::take(&mut new_key),
                                        toml::Value::String(String::new()),
                                    );
                                }
                                Either::Right(v) => {
                                    v.insert(
                                        std::mem::take(&mut new_key),
                                        toml::Value::String(String::new()),
                                    );
                                }
                            }
                        }
                    });
                });

                if let Some(to_remove) = to_remove {
                    match table {
                        Either::Left(v) => {
                            v.remove(&to_remove);
                        }
                        Either::Right(v) => {
                            v.remove(&to_remove);
                        }
                    }
                    changed.push(format!("{path}{to_remove}"));
                }
            });

        ui.memory_mut(|m| {
            m.data.insert_persisted(id, new_key);
        });

        changed
    }
    fn show_position_data(ui: &mut egui::Ui, nodes: &[PlaNode]) {
        egui_extras::TableBuilder::new(ui)
            .id_salt("component position data")
            .columns(egui_extras::Column::auto().at_least(50.0), 4)
            .cell_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT))
            .header(10.0, |mut header| {
                header.col(|_| ());
                header.col(|ui| {
                    ui.label("X");
                });
                header.col(|ui| {
                    ui.label("Y");
                });
            })
            .body(|mut body| {
                let mut add_row =
                    |ty: &str, coord: geo::Coord<i32>, colour: egui::Color32, label: Option<u8>| {
                        body.row(10.0, |mut row| {
                            row.col(|ui| {
                                ui.label(ty);
                            });
                            row.col(|ui| {
                                ui.label(
                                    egui::RichText::new(coord.x.to_string())
                                        .color(colour)
                                        .monospace(),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    egui::RichText::new(coord.y.to_string())
                                        .color(colour)
                                        .monospace(),
                                );
                            });
                            if let Some(label) = label {
                                row.col(|ui| {
                                    ui.label(format!("@ {label}"));
                                });
                            }
                        });
                    };
                for node in nodes {
                    match *node {
                        PlaNode::Line { coord, label } => {
                            add_row("line", coord, egui::Color32::WHITE, label);
                        }
                        PlaNode::QuadraticBezier { ctrl, coord, label } => {
                            add_row("ctrl", ctrl, egui::Color32::DARK_GRAY, None);
                            add_row("quad", coord, egui::Color32::WHITE, label);
                        }
                        PlaNode::CubicBezier {
                            ctrl1,
                            ctrl2,
                            coord,
                            label,
                        } => {
                            add_row("ctrl1", ctrl1, egui::Color32::DARK_GRAY, None);
                            add_row("ctrl2", ctrl2, egui::Color32::DARK_GRAY, None);
                            add_row("cubic", coord, egui::Color32::WHITE, label);
                        }
                    }
                }
            });
    }
}
