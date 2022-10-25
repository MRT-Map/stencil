use bevy::prelude::*;

mod changelog;
mod file;
mod info;
mod quit;

#[macro_export]
macro_rules! action {
    ($events:expr; $($id:literal, $ty:ty, $f:expr);+) => {{
        for event in $events.iter() {
            $(if event.id == $id {
                let payload = event.payload.downcast_ref::<$ty>().unwrap();
                $f(payload)
            })else+
        }
    }};

}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(quit::quit_msy)
            .add_system(info::info_msy)
            .add_system(changelog::changelog_msy)
            .add_system(file::load_ns::load_ns_msy.exclusive_system())
            .add_system(file::save_ns::save_ns_msy.exclusive_system());
    }
}
