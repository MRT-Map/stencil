use std::{collections::BTreeSet, path::PathBuf, sync::Arc};

use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    misc::Action,
    pla2::{
        bundle::ComponentBundle,
        component::{EditorCoords, MCCoords, PlaComponent},
        skin::Skin,
    },
    ui::{file_explorer::open_multiple_files, popup::Popup},
    EventReader,
};

pub fn load_ns_asy(
    mut actions: ParamSet<(EventReader<Action>, EventWriter<Action>)>,
    mut popup: EventWriter<Arc<Popup>>,
    mut commands: Commands,
    skin: Res<Skin>,
    existing_comps: Query<(&PlaComponent<EditorCoords>, Entity)>,
) {
    let mut send_queue: Vec<Action> = vec![];
    for event in actions.p0().iter() {
        if event.id == "load_ns" {
            open_multiple_files("load_ns1", &mut popup);
        } else if event.id == "load_ns1" {
            let files: &BTreeSet<PathBuf> = if let Some(files) = event
                .payload
                .downcast_ref::<Option<BTreeSet<PathBuf>>>()
                .unwrap()
            {
                files
            } else {
                continue;
            };
            let existing_namespaces: Arc<BTreeSet<String>> = Arc::new(
                existing_comps
                    .iter()
                    .map(|(a, _)| a.namespace.to_owned())
                    .sorted()
                    .dedup()
                    .collect::<BTreeSet<_>>(),
            );
            for file in files {
                send_queue.push(Action {
                    id: "load_ns2".into(),
                    payload: Box::new((file.to_owned(), existing_namespaces.to_owned())),
                })
            }
        } else if event.id == "load_ns2" {
            let (file, existing_namespaces): &(PathBuf, Arc<BTreeSet<String>>) =
                event.payload.downcast_ref().unwrap();
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
                        content,
                    ));
                    continue;
                }
            }
            send_queue.push(Action {
                id: "load_ns3".into(),
                payload: Box::new(content),
            });
        } else if event.id == "load_ns3" {
            let content: &Vec<PlaComponent<MCCoords>> = event.payload.downcast_ref().unwrap();
            if content.is_empty() {
                // TODO
            }
            for comp in content {
                let mut bundle = ComponentBundle::new(comp.to_editor_coords());
                bundle.update_shape(&skin);
                commands.spawn_bundle(bundle);
            }
            popup.send(Popup::base_alert(
                format!("load_ns_success_{}", content[0].namespace),
                "Loaded",
                format!("Successfully loaded {}", content[0].namespace),
            ))
        }
    }
    for action in send_queue {
        actions.p1().send(action)
    }
}
