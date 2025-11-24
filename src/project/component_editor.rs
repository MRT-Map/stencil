use std::{collections::BTreeMap, sync::Arc};

use egui::Widget;
use itertools::{Either, Itertools};
use serde::{Deserialize, Serialize};

use crate::{
    App,
    component_actions::ComponentEv,
    project::{
        pla3::{PlaComponent, PlaNode},
        skin::SkinType,
    },
    ui::dock::DockWindow,
};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ComponentEditorWindow;

impl DockWindow for ComponentEditorWindow {
    fn title(&self) -> String {
        "Component".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        let Some(skin) = app.project.skin() else {
            ui.heading("Waiting for skin...");
            return;
        };
        let mut selected_components = app
            .ui
            .dock_layout
            .map_window()
            .selected_components_mut(&mut app.project.components);
        if selected_components.is_empty() {
            ui.heading("Select components...");
            return;
        }

        let mut events_to_add = Vec::new();
        let mut old_selected_components = selected_components
            .iter()
            .map(|a| (**a).clone())
            .collect::<Vec<_>>();
        let mut add_event = |label: &'static str, selected_components: &[&mut PlaComponent]| {
            let new_components = selected_components
                .iter()
                .map(|a| (**a).clone())
                .collect::<Vec<_>>();
            events_to_add.push(ComponentEv::ChangeField {
                before: std::mem::take(&mut old_selected_components),
                after: new_components.clone(),
                label,
            });
            old_selected_components = new_components;
        };

        ui.heading("Edit component data");
        ui.end_row();

        ui.horizontal(|ui| {
            let namespace = selected_components
                .iter()
                .map(|c| &c.full_id.namespace)
                .sorted()
                .dedup()
                .exactly_one()
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

            if let Ok(component) = selected_components.iter_mut().exactly_one() {
                ui.code(&component.full_id.id);
            } else {
                ui.label(egui::RichText::new("mixed ids").italics());
            }
        });
        ui.end_row();

        let display_name = selected_components
            .iter()
            .map(|c| &c.display_name)
            .sorted()
            .dedup()
            .exactly_one()
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

        let skin_ty = selected_components
            .iter()
            .map(|c| &c.ty)
            .sorted_by_key(|a| a.name())
            .dedup()
            .exactly_one()
            .map(Arc::clone)
            .ok();
        let component_ty = selected_components
            .iter()
            .map(|c| &c.ty)
            .map(|a| match &**a {
                SkinType::Point { .. } => "point",
                SkinType::Line { .. } => "line",
                SkinType::Area { .. } => "area",
            })
            .dedup()
            .exactly_one()
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

        let layer = selected_components
            .iter()
            .map(|c| c.layer)
            .sorted_by(f32::total_cmp)
            .dedup()
            .exactly_one()
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

        let Ok(component) = selected_components.iter_mut().exactly_one() else {
            return;
        };
        ui.heading("Other Attributes");

        #[expect(clippy::items_after_statements)]
        fn field_editor(ui: &mut egui::Ui, v: &mut toml::Value, path: &str) {
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
                        }
                        if ui
                            .selectable_label(matches!(v, toml::Value::Integer(_)), "Integer")
                            .clicked()
                        {
                            *v = toml::Value::Integer(0);
                        }
                        if ui
                            .selectable_label(matches!(v, toml::Value::Float(_)), "Float")
                            .clicked()
                        {
                            *v = toml::Value::Float(0.0);
                        }
                        if ui
                            .selectable_label(matches!(v, toml::Value::Boolean(_)), "Boolean")
                            .clicked()
                        {
                            *v = toml::Value::Boolean(false);
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
                        }
                        if ui
                            .selectable_label(matches!(v, toml::Value::Array(_)), "Array")
                            .clicked()
                        {
                            *v = toml::Value::Array(Vec::new());
                        }
                        if ui
                            .selectable_label(matches!(v, toml::Value::String(_)), "Table")
                            .clicked()
                        {
                            *v = toml::Value::Table(toml::Table::new());
                        }
                    });

                match v {
                    toml::Value::String(v) => {
                        ui.text_edit_multiline(v);
                    }
                    toml::Value::Integer(v) => {
                        ui.add(egui::DragValue::new(v));
                    }
                    toml::Value::Float(v) => {
                        ui.add(egui::DragValue::new(v));
                    }
                    toml::Value::Boolean(v) => {
                        ui.checkbox(v, "");
                    }
                    toml::Value::Datetime(_) | toml::Value::Array(_) | toml::Value::Table(_) => {}
                }
            });

            match v {
                toml::Value::Array(v) => array_editor(ui, v, &format!("{path}/")),
                toml::Value::Table(v) => table_editor(ui, Either::Right(v), &format!("{path}/")),
                toml::Value::Datetime(v) => {
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
                }
                _ => {}
            }
        }

        #[expect(clippy::items_after_statements)]
        fn array_editor(ui: &mut egui::Ui, array: &mut Vec<toml::Value>, path: &str) {
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
                    field_editor(ui, v, &format!("{path}/{i}/"));
                });
            }
            if ui
                .add(egui::Button::new("➕").right_text("Add to array"))
                .clicked()
            {
                array.push(toml::Value::String(String::new()));
            }

            if let Some(to_remove) = to_remove {
                array.remove(to_remove);
            }
            if let Some((a, b)) = to_swap {
                array.swap(a, b);
            }
        }

        #[expect(clippy::items_after_statements)]
        fn table_editor(
            ui: &mut egui::Ui,
            mut table: Either<&mut BTreeMap<String, toml::Value>, &mut toml::Table>,
            path: &str,
        ) {
            let mut new_key = ui.memory_mut(|m| {
                m.data
                    .get_persisted::<String>(format!("{path} new key").into())
                    .unwrap_or_default()
            });

            egui_extras::TableBuilder::new(ui)
                .id_salt(format!("{path} table"))
                .striped(true)
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::remainder())
                .body(|mut body| {
                    let mut to_remove = None;

                    let iter: Box<dyn Iterator<Item = (&String, &mut toml::Value)>> =
                        match &mut table {
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
                                field_editor(ui, v, &format!("{path}{k}"));
                            });
                        });
                    }

                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut new_key).hint_text("Add to table"),
                            );
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
                    }
                });

            ui.memory_mut(|m| {
                m.data
                    .insert_persisted(format!("{path} new key").into(), new_key);
            });
        }

        table_editor(ui, Either::Left(&mut component.misc), "component ");

        ui.heading("Position data");
        let is_line = matches!(&*component.ty, SkinType::Line { .. });
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
                for (i, node) in component.nodes.iter().enumerate() {
                    let colour = if i == 0 && is_line {
                        egui::Color32::GREEN
                    } else if i == component.nodes.len() - 1 && is_line {
                        egui::Color32::RED
                    } else {
                        egui::Color32::WHITE
                    };
                    match *node {
                        PlaNode::Line { coord, label } => {
                            add_row("line", coord, colour, label);
                        }
                        PlaNode::QuadraticBezier { ctrl, coord, label } => {
                            add_row("ctrl", ctrl, egui::Color32::WHITE, None);
                            add_row("quad", coord, colour, label);
                        }
                        PlaNode::CubicBezier {
                            ctrl1,
                            ctrl2,
                            coord,
                            label,
                        } => {
                            add_row("ctrl1", ctrl1, egui::Color32::WHITE, None);
                            add_row("ctrl2", ctrl2, egui::Color32::WHITE, None);
                            add_row("cubic", coord, colour, label);
                        }
                    }
                }
            });

        for ev in events_to_add {
            app.add_event(ev);
        }
    }
}
