use bevy::prelude::*;
use native_dialog::{FileDialog, MessageDialog, MessageType};

use crate::{
    editor::{bundles::component::ComponentBundle, menu_actions::MenuAction},
    menu,
    types::{
        pla::{MCCoords, PlaComponent},
        skin::Skin,
    },
    EventReader,
};

pub fn load_ns_msy(mut events: EventReader<MenuAction>, mut commands: Commands, skin: Res<Skin>) {
    menu!(events, "load_ns");
    let files = FileDialog::default().show_open_multiple_file().unwrap();
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
        for comp in content {
            let mut bundle = ComponentBundle::new(comp.to_editor_coords());
            bundle.update_shape(&skin);
            commands.spawn_bundle(bundle);
        }
    }
}
