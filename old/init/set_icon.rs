use bevy::{prelude::*, window::PrimaryWindow, winit::WinitWindows};
use eyre::OptionExt;
use winit::window::Icon;

use crate::state::LoadingState;

#[expect(clippy::needless_pass_by_value)]
pub fn set_icon_sy(
    mut commands: Commands,
    windows: NonSendMut<WinitWindows>,
    primary_id: Query<Entity, With<PrimaryWindow>>,
) -> Result {
    info!("Setting the window icon");
    // https://bevy-cheatbook.github.io/window/icon.html
    let primary = windows
        .get_window(primary_id.single()?)
        .ok_or_eyre("no window")?;

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(include_bytes!("../../icons/icon_512x512@2x.png"))?
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height)?;

    primary.set_window_icon(Some(icon));

    commands.insert_resource(NextState::Pending(LoadingState::SetIcon.next()));
    Ok(())
}
