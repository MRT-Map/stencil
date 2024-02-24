use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    load_save::LoadSaveAct,
    misc::Action,
    pla2::component::{EditorCoords, MCCoords, PlaComponent},
    ui::{file_explorer::save_single_dir, popup::Popup},
};

#[allow(clippy::needless_pass_by_value)]
pub fn save_ns_asy(
    mut actions: EventReader<Action>,
    query: Query<&PlaComponent<EditorCoords>>,
    mut popup: EventWriter<Popup>,
) {
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(LoadSaveAct::Save)) {
            save_single_dir("save_ns1", &mut popup, |a| {
                Action::new(LoadSaveAct::Save1(a))
            });
        } else if let Some(LoadSaveAct::Save1(Some(dir))) = event.downcast_ref() {
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
                    .push(comp.to_mc_coords());
            }
            for (ns, comps) in &files {
                info!(?ns, "Saving namespace");
                let fp = dir.join(PathBuf::from(format!("{ns}.pla2.msgpack")));
                std::fs::write(fp, rmp_serde::to_vec_named(comps).unwrap()).unwrap();
            }
            popup.send(Popup::base_alert(
                "save_ns_success",
                "Components saved!",
                format!("Namespaces: {}", files.keys().join(", ")),
            ));
        }
    }
}
