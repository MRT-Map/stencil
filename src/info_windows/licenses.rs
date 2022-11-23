use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::egui;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{misc::Action, ui::popup::Popup};

#[derive(Deserialize, Serialize, Clone)]
pub struct CargoLicenseEntry {
    name: String,
    version: String,
    authors: Option<String>,
    repository: Option<String>,
    license: Option<String>,
    license_file: Option<String>,
    license_text: Option<Vec<String>>,
}

static LICENSES: Lazy<Vec<CargoLicenseEntry>> = Lazy::new(|| {
    rmp_serde::from_slice(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/licenses.msgpack"
    )))
    .unwrap()
});

pub fn licenses_asy(mut actions: EventReader<Action>, mut popup: EventWriter<Arc<Popup>>) {
    for event in actions.iter() {
        if event.id == "licenses" {
            popup.send(Popup::new(
                "info_popup",
                || {
                    egui::Window::new("Open Source Licenses")
                        .collapsible(true)
                        .resizable(true)
                        .vscroll(true)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                },
                |state, ui, _, shown| {
                    let mut state = state.lock().unwrap();
                    let selection: &mut (String, String) = state.downcast_mut().unwrap();
                    egui::ComboBox::from_label("Library")
                        .selected_text(format!("{} {}", selection.0, selection.1))
                        .show_ui(ui, |ui| {
                            LICENSES.iter().for_each(|entry| {
                                ui.selectable_value(
                                    selection,
                                    (entry.name.to_owned(), entry.version.to_owned()),
                                    format!("{} {}", entry.name, entry.version),
                                );
                            });
                        });
                    let entry = LICENSES
                        .iter()
                        .find(|a| a.name == selection.0 && a.version == selection.1)
                        .unwrap()
                        .to_owned();
                    ui.heading(format!("{} v{}", entry.name, entry.version));
                    ui.label(format!(
                        "by: {}",
                        entry.authors.unwrap_or_else(|| "unknown".into())
                    ));
                    ui.label(format!(
                        "is licensed under: {}",
                        entry.license.unwrap_or_else(|| "unknown".into())
                    ));
                    if let Some(repo) = entry.repository {
                        ui.hyperlink(repo);
                    }
                    for text in entry.license_text.unwrap_or_default() {
                        ui.separator();
                        ui.label(text);
                    }
                    ui.separator();
                    if ui.button("Close").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new((
                    "stencil2".to_string(),
                    env!("CARGO_PKG_VERSION").to_string(),
                ))),
            ));
        }
    }
}
