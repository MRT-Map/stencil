use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::egui;
use license_retriever::LicenseRetriever;
use once_cell::sync::Lazy;

use crate::{info_windows::InfoWindowsAct, misc::Action, ui::popup::Popup};

#[cfg(not(debug_assertions))]
static LICENSES: Lazy<LicenseRetriever> =
    Lazy::new(|| license_retriever::license_retriever_data!("licenses").unwrap());

#[cfg(debug_assertions)]
static LICENSES: Lazy<LicenseRetriever> = Lazy::new(|| LicenseRetriever::default());

pub fn licenses_asy(mut actions: EventReader<Action>, mut popup: EventWriter<Arc<Popup>>) {
    for event in actions.iter() {
        if matches!(event.downcast_ref(), Some(InfoWindowsAct::Licenses)) {
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
                    if cfg!(debug_assertions) {
                        *shown = false;
                        return;
                    }
                    egui::ComboBox::from_label("Library")
                        .selected_text(format!("{} {}", selection.0, selection.1))
                        .show_ui(ui, |ui| {
                            LICENSES.iter().for_each(|(package, _)| {
                                ui.selectable_value(
                                    selection,
                                    (package.name.to_owned(), package.version.to_string()),
                                    format!("{} {}", package.name, package.version),
                                );
                            });
                        });
                    let (entry, licenses) = LICENSES
                        .iter()
                        .find(|(p, _)| {
                            p.name == selection.0 && p.version.to_string() == selection.1
                        })
                        .unwrap()
                        .to_owned();
                    ui.heading(format!("{} v{}", entry.name, entry.version));
                    ui.label(format!("by: {}", entry.authors.join(", ")));
                    ui.label(format!(
                        "is licensed under: {}",
                        entry.license.unwrap_or_else(|| "unknown".into())
                    ));
                    if let Some(repo) = entry.repository {
                        ui.hyperlink(repo);
                    }
                    for text in licenses.as_ref() {
                        ui.separator();
                        ui.label(text);
                    }
                    ui.separator();
                    if ui.button("Close").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new((
                    "stencil2".to_owned(),
                    env!("CARGO_PKG_VERSION").to_owned(),
                ))),
            ));
        }
    }
}
