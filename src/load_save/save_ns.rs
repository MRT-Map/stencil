use std::{collections::HashMap, path::PathBuf, sync::Arc};

use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    misc::Action,
    pla2::component::{EditorCoords, MCCoords, PlaComponent},
    ui::{file_explorer::save_single_dir, popup::Popup},
    EventReader,
};

pub fn save_ns_asy(
    mut actions: EventReader<Action>,
    query: Query<&PlaComponent<EditorCoords>>,
    mut popup: EventWriter<Arc<Popup>>,
) {
    for event in actions.iter() {
        if event.id == "save_ns" {
            save_single_dir("save_ns1", &mut popup);
        } else if event.id == "save_ns1" {
            let dir = if let Some(dir) = event.payload.downcast_ref::<Option<PathBuf>>().unwrap() {
                dir
            } else {
                continue;
            };
            let comps = query.iter().collect::<Vec<_>>();
            let mut files: HashMap<&String, Vec<PlaComponent<MCCoords>>> = HashMap::new();
            for comp in comps {
                if comp.namespace.is_empty() {
                    popup.send(Popup::base_alert(
                        "save_ns_err",
                        "Empty namespace detected!",
                        format!("It is at {}, {}", comp.nodes[0].0.x, comp.nodes[0].0.y),
                    ));
                    continue;
                }
                files
                    .entry(&comp.namespace)
                    .or_default()
                    .push(comp.to_mc_coords())
            }
            for (ns, comps) in files.iter() {
                info!(?ns, "Saving namespace");
                let mut fp = dir.to_owned();
                fp.push(PathBuf::from(format!("{ns}.pla2.msgpack")));
                std::fs::write(fp, rmp_serde::to_vec_named(comps).unwrap()).unwrap();
            }
            popup.send(Popup::base_alert(
                "save_ns_success",
                "Components saved!",
                format!("Namespaces: {}", files.keys().join(", ")),
            ))
        }
    }
}
