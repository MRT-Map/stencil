use std::sync::{LazyLock, Mutex};

use bevy::prelude::*;
use bevy_egui::egui;
use itertools::Itertools;
use license_retriever::LicenseRetriever;

use crate::{
    info_windows::InfoWindowsEv,
    ui::popup::{Popup, Popups},
};

#[cfg(not(debug_assertions))]
static LICENSES: LazyLock<LicenseRetriever> =
    LazyLock::new(|| license_retriever::license_retriever_data!("licenses").unwrap());

#[cfg(debug_assertions)]
static LICENSES: LazyLock<LicenseRetriever> = LazyLock::new(LicenseRetriever::default);

#[expect(clippy::needless_pass_by_value, clippy::significant_drop_tightening)]
pub fn on_license(trigger: Trigger<InfoWindowsEv>, mut popups: ResMut<Popups>) {
    if *trigger.event() != InfoWindowsEv::Licenses {
        return;
    }
    popups.add(Popup::new(
        "licenses",
        || {
            egui::Window::new("Open Source Licenses")
                .collapsible(true)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        },
        |state, ui, _, shown| {
            let mut state = state.lock().unwrap();
            let selection: &mut (String, String) = state.downcast_mut().unwrap();

            egui::ComboBox::from_label("Library")
                .selected_text(format!("{} {}", selection.0, selection.1))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    LICENSES
                        .iter()
                        .sorted_by_key(|(package, _)| (&package.name, &package.version))
                        .for_each(|(package, _)| {
                            ui.selectable_value(
                                selection,
                                (package.name.to_string(), package.version.to_string()),
                                format!("{} {}", package.name, package.version),
                            );
                        });
                });
            let Some((entry, licenses)) = LICENSES.iter().find(|(p, _)| {
                p.name.as_str() == selection.0 && p.version.to_string() == selection.1
            }) else {
                ui.label("Invalid selection");
                if ui.button("Close").clicked() {
                    *shown = false;
                }
                return;
            };
            ui.heading(format!("{} v{}", entry.name, entry.version));
            ui.label(format!("by: {}", entry.authors.join(", ")));
            ui.label(format!(
                "is licensed under: {}",
                entry.license.clone().unwrap_or_else(|| "unknown".into())
            ));
            if let Some(repo) = &entry.repository {
                ui.horizontal(|ui| {
                    ui.label("Repository:");
                    ui.hyperlink(repo);
                });
            }
            if let Some(home) = &entry.homepage {
                ui.horizontal(|ui| {
                    ui.label("Homepage:");
                    ui.hyperlink(home);
                });
            }
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(ui.available_height() * 0.75)
                .show(ui, |ui| {
                    for text in licenses {
                        ui.label(text);
                        ui.separator();
                    }
                });
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
