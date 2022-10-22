use std::collections::HashSet;

use bevy::prelude::*;
use itertools::Itertools;
use native_dialog::{FileDialog, MessageDialog, MessageType};

use crate::{
    editor::{bundles::component::ComponentBundle, menu_actions::MenuAction},
    menu,
    types::{
        pla::{EditorCoords, MCCoords, PlaComponent},
        skin::Skin,
    },
    EventReader,
};

pub fn load_ns_msy(
    mut events: EventReader<MenuAction>,
    mut commands: Commands,
    skin: Res<Skin>,
    existing_comps: Query<(&PlaComponent<EditorCoords>, Entity)>,
) {
    menu!(events, "load_ns");
    let files = FileDialog::default().show_open_multiple_file().unwrap();
    let existing_namespaces: HashSet<&String> = existing_comps
        .iter()
        .map(|(a, _)| &a.namespace)
        .sorted()
        .dedup()
        .collect::<HashSet<_>>();
    for file in &files {
        let bytes = std::fs::read(file).unwrap();
        let content: Vec<PlaComponent<MCCoords>> = match rmp_serde::from_slice(&bytes) {
            Ok(res) => res,
            Err(err) => {
                MessageDialog::default()
                    .set_title(&format!("Error parsing {}", file.display()))
                    .set_text(&format!("Error: {err}"))
                    .set_type(MessageType::Error)
                    .show_alert()
                    .unwrap();
                continue;
            }
        };
        if let Some(first) = content.first() {
            if existing_namespaces.contains(&first.namespace) {
                if MessageDialog::default()
                    .set_title(&format!(
                        "The namespace {} is already loaded.",
                        first.namespace
                    ))
                    .set_text("Do you want to override this namespace?")
                    .set_type(MessageType::Warning)
                    .show_confirm()
                    .unwrap()
                {
                    existing_comps
                        .iter()
                        .filter(|(a, _)| a.namespace == first.namespace)
                        .map(|(_, a)| a)
                        .for_each(|a| commands.entity(a).despawn_recursive())
                } else {
                    continue;
                }
            }
        }
        for comp in content {
            let mut bundle = ComponentBundle::new(comp.to_editor_coords());
            bundle.update_shape(&skin);
            commands.spawn_bundle(bundle);
        }
    }
    MessageDialog::default()
        .set_title("Components loaded")
        .set_text(&format!(
            "Namespaces: {}",
            files
                .iter()
                .filter_map(|f| Some(
                    f.file_stem()?
                        .to_string_lossy()
                        .split('.')
                        .next()?
                        .to_owned()
                ))
                .join(", ")
        ))
        .set_type(MessageType::Info)
        .show_alert()
        .unwrap();
}
