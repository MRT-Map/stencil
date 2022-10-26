use std::{
    collections::{BTreeSet, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bevy::prelude::EventWriter;
use bevy_egui::{egui, egui::Color32};
use egui_extras::{Size, StripBuilder, TableBuilder};
use itertools::Itertools;

use crate::{misc::Action, ui::popup::Popup};

pub fn file_explorer(
    ui: &mut egui::Ui,
    current_path: &mut PathBuf,
    mut chosen_files: Option<&mut HashSet<PathBuf>>,
) {
    let files: BTreeSet<PathBuf> = current_path
        .read_dir()
        .and_then(|rd| rd.into_iter().map_ok(|rd| rd.path()).collect())
        .unwrap_or_default();
    if ui.button("Back").clicked() {
        *current_path = current_path
            .parent()
            .unwrap_or(current_path.as_path())
            .into()
    }
    StripBuilder::new(ui)
        .size(Size::relative(0.75))
        .vertical(|mut strip| {
            strip.cell(|ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Size::relative(0.05).at_least(0.05))
                    .column(Size::remainder())
                    .scroll(true)
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("");
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
                                            chosen_files.insert(file.to_owned());
                                        } else {
                                            chosen_files.remove(&file);
                                        }
                                    } else {
                                        ui.label("");
                                    }
                                });
                                row.col(|ui| {
                                    if file.is_dir()
                                        && ui
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
                                        *current_path = file;
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
            })
        });
}

pub fn open_multiple_files(
    id: impl std::fmt::Display + Sync + Send + 'static,
    popup: &mut EventWriter<Arc<Popup>>,
) {
    popup.send(Popup::new(
        id.to_string(),
        || egui::Window::new("Opening multiple files").resizable(true),
        move |state, ui, ew, shown| {
            let mut state = state.lock().unwrap();
            let (current_path, chosen_files): &mut (PathBuf, HashSet<PathBuf>) =
                state.downcast_mut().unwrap();
            file_explorer(ui, current_path, Some(chosen_files));
            if ui.button("Select").clicked() {
                ew.send(Action {
                    id: id.to_string(),
                    payload: Box::new(Some(chosen_files.to_owned())),
                });
                *shown = false;
            }
            if ui.button("Cancel").clicked() {
                ew.send(Action {
                    id: id.to_string(),
                    payload: Box::new(Option::<HashSet<PathBuf>>::None),
                });
                *shown = false;
            }
        },
        Mutex::new(Box::new((PathBuf::from("/"), HashSet::<PathBuf>::new()))),
    ))
}

pub fn save_single_dir(
    id: impl std::fmt::Display + Sync + Send + 'static,
    popup: &mut EventWriter<Arc<Popup>>,
) {
    popup.send(Popup::new(
        id.to_string(),
        || egui::Window::new("Saving to a directory").resizable(true),
        move |state, ui, ew, shown| {
            let mut state = state.lock().unwrap();
            let current_path: &mut PathBuf = state.downcast_mut().unwrap();
            file_explorer(ui, current_path, None);
            if ui.button("Select").clicked() {
                ew.send(Action {
                    id: id.to_string(),
                    payload: Box::new(Some(current_path.to_owned())),
                });
                *shown = false;
            }
            if ui.button("Cancel").clicked() {
                ew.send(Action {
                    id: id.to_string(),
                    payload: Box::new(Option::<PathBuf>::None),
                });
                *shown = false;
            }
        },
        Mutex::new(Box::new(PathBuf::from("/"))),
    ))
}
