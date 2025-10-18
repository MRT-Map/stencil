use std::{
    fmt::{Display, Formatter},
    sync::LazyLock,
};

use itertools::Itertools;
use license_retriever::LicenseRetriever;
use serde::{Deserialize, Serialize};

use crate::{App, ui::popup::Popup};

#[cfg(not(debug_assertions))]
static LICENSES: LazyLock<LicenseRetriever> =
    LazyLock::new(|| license_retriever::license_retriever_data!("licenses").unwrap());

#[cfg(debug_assertions)]
static LICENSES: LazyLock<LicenseRetriever> = LazyLock::new(LicenseRetriever::default);

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
struct SelectedPackage {
    name: String,
    version: String,
}

impl Display for SelectedPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LicensesPopup {
    selected: SelectedPackage,
}

impl Default for LicensesPopup {
    fn default() -> Self {
        Self {
            selected: SelectedPackage {
                name: "stencil2".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
        }
    }
}

impl Popup for LicensesPopup {
    fn id(&self) -> String {
        "licenses".into()
    }

    fn title(&self) -> String {
        "Licenses".into()
    }

    fn window(&self) -> egui::Window<'static> {
        self.default_window().collapsible(true).resizable(true)
    }

    fn ui(&mut self, _app: &mut App, ui: &mut egui::Ui) -> bool {
        egui::ComboBox::from_label("Library")
            .selected_text(format!("{}", self.selected))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                LICENSES
                    .iter()
                    .sorted_by_key(|(package, _)| (&package.name, &package.version))
                    .for_each(|(package, _)| {
                        let package_value = SelectedPackage {
                            name: package.name.to_string(),
                            version: package.version.to_string(),
                        };
                        let display = format!("{package_value}");
                        ui.selectable_value(&mut self.selected, package_value, display);
                    });
            });

        let Some((entry, licenses)) = LICENSES.iter().find(|(p, _)| {
            p.name.as_str() == self.selected.name && p.version.to_string() == self.selected.version
        }) else {
            ui.label("Invalid selection");
            ui.separator();
            return !ui.button("Close").clicked();
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
        !ui.button("Close").clicked()
    }
}
