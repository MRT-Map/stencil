use bevy::prelude::*;

mod file;
mod quit;

pub struct MenuAction(pub &'static str);

#[macro_export]
macro_rules! menu {
    ($events:ident, $id:literal) => {{
        let mut will_run = false;
        for event in $events.iter() {
            if event.0 == $id {
                will_run = true;
            }
        }
        if !will_run {
            return;
        }
    }};
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MenuAction>()
            .add_system(quit::quit_msy)
            .add_system(file::load_ns::load_ns_msy.exclusive_system());
    }
}
