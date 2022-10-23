use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;
use itertools::Itertools;
use native_dialog::{FileDialog, MessageDialog, MessageType};

use crate::{
    menu,
    menu_actions::MenuAction,
    pla2::component::{EditorCoords, MCCoords, PlaComponent},
    EventReader,
};

pub fn save_ns_msy(mut events: EventReader<MenuAction>, query: Query<&PlaComponent<EditorCoords>>) {
    menu!(events, "save_ns");
    let comps = query.iter().collect::<Vec<_>>();
    let mut files: HashMap<&String, Vec<PlaComponent<MCCoords>>> = HashMap::new();
    for comp in comps {
        if comp.namespace.is_empty() {
            MessageDialog::default()
                .set_title("Empty namespace detected!")
                .set_text(&format!(
                    "It is at {}, {}",
                    comp.nodes[0].0.x, comp.nodes[0].0.y
                ))
                .set_type(MessageType::Error)
                .show_alert()
                .unwrap();
            return;
        }
        files
            .entry(&comp.namespace)
            .or_default()
            .push(comp.to_mc_coords())
    }
    let dir = if let Some(dir) = FileDialog::default().show_open_single_dir().unwrap() {
        dir
    } else {
        return;
    };
    for (ns, comps) in files.iter() {
        let mut fp = dir.to_owned();
        fp.push(PathBuf::from(format!("{ns}.pla2.msgpack")));
        std::fs::write(fp, rmp_serde::to_vec_named(comps).unwrap()).unwrap();
    }
    MessageDialog::default()
        .set_title("Components saved!")
        .set_text(&format!("Namespaces: {}", files.keys().join(", ")))
        .set_type(MessageType::Info)
        .show_alert()
        .unwrap();
}
