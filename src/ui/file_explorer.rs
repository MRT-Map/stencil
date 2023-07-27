use std::{collections::BTreeSet, path::PathBuf, sync::Mutex};

use bevy::prelude::*;
use bevy_egui::{egui, egui::Color32};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use itertools::Itertools;

use crate::{misc::Action, ui::popup::Popup};

#[tracing::instrument(skip_all)]
pub fn file_explorer(
    ui: &mut egui::Ui,
    current_path: &mut PathBuf,
    mut chosen_files: Option<&mut BTreeSet<PathBuf>>,
) {
    let files: BTreeSet<PathBuf> = current_path
        .read_dir()
        .and_then(|rd| {
            rd.into_iter()
                .map_ok(|rd| rd.path())
                .filter_ok(|p| !p.ends_with(PathBuf::from(".DS_Store")))
                .collect()
        })
        .unwrap_or_default();
    if ui.button("Back").clicked() {
        info!("Moving back one level");
        *current_path = current_path
            .parent()
            .unwrap_or(current_path.as_path())
            .into();
    }
    StripBuilder::new(ui)
        .size(Size::relative(0.75))
        .vertical(|mut strip| {
            strip.cell(|ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::auto().at_least(0.05))
                    .column(Column::remainder())
                    .vscroll(true)
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            if let Some(chosen_files) = &mut chosen_files {
                                let mut files_not_dir: BTreeSet<_> =
                                    files.iter().filter(|a| !a.is_dir()).cloned().collect();
                                let mut checked = files_not_dir.is_subset(chosen_files)
                                    && !files_not_dir.is_empty();
                                let old_checked = checked;
                                ui.checkbox(&mut checked, "");
                                if checked && checked != old_checked {
                                    info!(?files_not_dir, "Adding files to chosen file list");
                                    chosen_files.append(&mut files_not_dir);
                                } else if checked != old_checked {
                                    **chosen_files =
                                        chosen_files.difference(&files_not_dir).cloned().collect();
                                }
                            }
                        });
                        header.col(|ui| {
                            ui.heading(current_path.to_string_lossy());
                        });
                    })
                    .body(|mut body| {
                        for file in files {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    if file.is_dir() {
                                        ui.label("");
                                    } else if let Some(chosen_files) = &mut chosen_files {
                                        let mut checked = chosen_files.contains(&file);
                                        ui.checkbox(&mut checked, "");
                                        if checked {
                                            if !chosen_files.contains(&file) {
                                                info!(?file, "Adding file to chosen file list");
                                            }
                                            chosen_files.insert(file.to_owned());
                                        } else {
                                            if chosen_files.contains(&file) {
                                                info!(?file, "Removing file from chosen file list");
                                            }
                                            chosen_files.remove(&file);
                                        }
                                    } else {
                                        ui.label("");
                                    }
                                });
                                row.col(|ui| {
                                    if file.is_dir() {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(
                                                        file.file_name()
                                                            .unwrap_or_default()
                                                            .to_string_lossy(),
                                                    )
                                                    .color(Color32::WHITE),
                                                )
                                                .small()
                                                .wrap(false),
                                            )
                                            .clicked()
                                        {
                                            info!(?file, "Navigating to folder");
                                            *current_path = file;
                                        }
                                    } else if file.to_string_lossy().ends_with(".pla2.msgpack") {
                                        ui.label(
                                            egui::RichText::new(
                                                file.file_name()
                                                    .unwrap_or_default()
                                                    .to_string_lossy(),
                                            )
                                            .color(Color32::YELLOW),
                                        );
                                    } else {
                                        ui.label(
                                            egui::RichText::new(
                                                file.file_name()
                                                    .unwrap_or_default()
                                                    .to_string_lossy(),
                                            )
                                            .color(Color32::DARK_GRAY),
                                        );
                                    };
                                });
                            });
                        }
                    });
            });
        });
}

#[tracing::instrument(skip_all)]
pub fn open_multiple_files<
    I: std::fmt::Display + Sync + Send + 'static,
    F: FnOnce(Option<BTreeSet<PathBuf>>) -> Action + Send + Sync + Copy + 'static,
>(
    id: I,
    popup: &mut EventWriter<Popup>,
    action_fn: F,
) {
    popup.send(Popup::new(
        id.to_string(),
        || egui::Window::new("Opening multiple files").resizable(true),
        move |state, ui, ew, shown| {
            let mut state = state.lock().unwrap();
            let (current_path, chosen_files): &mut (PathBuf, BTreeSet<PathBuf>) =
                state.downcast_mut().unwrap();
            ui.label(egui::RichText::new("Selected:").strong());
            ui.label(format!(
                "Selected:\n{}",
                chosen_files
                    .iter()
                    .take(10)
                    .map(|a| a.to_string_lossy())
                    .join("\n ")
            ));
            if chosen_files.len() > 10 {
                ui.label("...");
            }
            file_explorer(ui, current_path, Some(chosen_files));
            ui.horizontal(|ui| {
                if ui.button("Select").clicked() {
                    info!("Files selected");
                    debug!(?chosen_files);
                    ew.send(action_fn(Some(chosen_files.to_owned())));
                    *shown = false;
                }
                if ui.button("Cancel").clicked() {
                    info!("Operation cancelled");
                    ew.send(action_fn(None));
                    *shown = false;
                }
            });
        },
        Mutex::new(Box::new((
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
            BTreeSet::<PathBuf>::new(),
        ))),
    ));
}

#[tracing::instrument(skip_all)]
pub fn save_single_dir<
    I: std::fmt::Display + Sync + Send + 'static,
    F: FnOnce(Option<PathBuf>) -> Action + Send + Sync + Copy + 'static,
>(
    id: I,
    popup: &mut EventWriter<Popup>,
    action_fn: F,
) {
    popup.send(Popup::new(
        id.to_string(),
        || egui::Window::new("Saving to a directory").resizable(true),
        move |state, ui, ew, shown| {
            let mut state = state.lock().unwrap();
            let current_path: &mut PathBuf = state.downcast_mut().unwrap();
            file_explorer(ui, current_path, None);
            ui.horizontal(|ui| {
                if ui.button("Select").clicked() {
                    info!("Folder selected");
                    debug!(?current_path);
                    ew.send(action_fn(Some(current_path.to_owned())));
                    *shown = false;
                }
                if ui.button("Cancel").clicked() {
                    info!("Operation cancelled");
                    ew.send(action_fn(None));
                    *shown = false;
                }
            });
        },
        Mutex::new(Box::new(
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        )),
    ));
}
