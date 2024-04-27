use std::{collections::BTreeSet, sync::Arc};

use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    component_actions::undo_redo::{History, UndoRedoAct},
    load_save::LoadSaveAct,
    misc::Action,
    pla2::{
        bundle::{AreaComponentBundle, LineComponentBundle, PointComponentBundle},
        component::{ComponentType, EditorCoords, MCCoords, PlaComponent},
        skin::Skin,
    },
    ui::{file_explorer::open_multiple_files, popup::Popup},
};

#[allow(clippy::needless_pass_by_value)]
pub fn load_ns_asy(
    mut actions: ParamSet<(EventReader<Action>, EventWriter<Action>)>,
    mut popup: EventWriter<Popup>,
    mut commands: Commands,
    skin: Res<Skin>,
    existing_comps: Query<(&PlaComponent<EditorCoords>, Entity)>,
) {
    let mut send_queue: Vec<Action> = vec![];
    for event in actions.p0().read() {
        if matches!(event.downcast_ref(), Some(LoadSaveAct::Load)) {
            open_multiple_files("load_ns1", &mut popup, |a| {
                Action::new(LoadSaveAct::Load1(a))
            });
        } else if let Some(LoadSaveAct::Load1(Some(files))) = event.downcast_ref() {
            let existing_namespaces: Arc<BTreeSet<String>> = Arc::new(
                existing_comps
                    .iter()
                    .map(|(a, _)| a.namespace.to_owned())
                    .sorted()
                    .dedup()
                    .collect::<BTreeSet<_>>(),
            );
            for file in files {
                send_queue.push(Action::new(LoadSaveAct::Load2(
                    file.to_owned(),
                    existing_namespaces.to_owned(),
                )));
            }
        } else if let Some(LoadSaveAct::Load2(file, existing_namespaces)) = event.downcast_ref() {
            info!(?file, "Reading file");
            let bytes = match std::fs::read(file) {
                Ok(bytes) => bytes,
                Err(err) => {
                    popup.send(Popup::base_alert(
                        format!("load_ns_err_{}", file.to_string_lossy()),
                        format!("Error loading {}", file.to_string_lossy()),
                        format!("Error: {err}"),
                    ));
                    continue;
                }
            };
            let content: Vec<PlaComponent<MCCoords>> = match rmp_serde::from_slice(&bytes) {
                Ok(res) => res,
                Err(err) => {
                    popup.send(Popup::base_alert(
                        format!("load_ns_err_{}", file.to_string_lossy()),
                        format!("Error parsing {}", file.to_string_lossy()),
                        format!("Error: {err}"),
                    ));
                    continue;
                }
            };
            if let Some(first) = content.first() {
                if existing_namespaces.contains(&first.namespace) {
                    popup.send(Popup::base_confirm(
                        "load_ns3",
                        format!("The namespace {} is already loaded.", first.namespace),
                        "Do you want to override this namespace?",
                        Action::new(LoadSaveAct::Load3(content)),
                    ));
                    continue;
                }
            }
            send_queue.push(Action::new(LoadSaveAct::Load3(content)));
        } else if let Some(LoadSaveAct::Load3(content)) = event.downcast_ref() {
            if content.is_empty() {
                popup.send(Popup::base_alert(
                    format!("load_ns_success_{}_empty", content[0].namespace),
                    "Loaded with no components",
                    format!("{} has no components", content[0].namespace),
                ));
            }
            let histories = content
                .iter()
                .map(|comp| {
                    let comp = comp.to_editor_coords();
                    let entity = match comp.get_type(&skin).unwrap() {
                        ComponentType::Point => {
                            commands.spawn(PointComponentBundle::new(comp.to_owned(), &skin))
                        }
                        ComponentType::Line => {
                            commands.spawn(LineComponentBundle::new(comp.to_owned(), &skin))
                        }
                        ComponentType::Area => {
                            commands.spawn(AreaComponentBundle::new(comp.to_owned(), &skin))
                        }
                    }
                    .id();
                    History {
                        component_id: entity,
                        before: None,
                        after: Some(comp),
                    }
                })
                .collect();
            send_queue.push(Action::new(UndoRedoAct::NewHistory(histories)));
            popup.send(Popup::base_alert(
                format!("load_ns_success_{}", content[0].namespace),
                "Loaded",
                format!("Successfully loaded {}", content[0].namespace),
            ));
        }
    }
    for action in send_queue {
        actions.p1().send(action);
    }
}
