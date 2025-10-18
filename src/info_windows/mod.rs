pub mod changelog;
pub mod info;
pub mod licenses;
pub mod manual;
pub mod quit;

use crate::{
    App,
    event::Event,
    info_windows::{
        changelog::ChangelogPopup, info::InfoPopup, licenses::LicensesPopup, manual::ManualPopup,
        quit::QuitPopup,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InfoWindowEv {
    Changelog,
    Info,
    Licenses,
    Manual,
    Quit { confirm: bool },
}

impl Event for InfoWindowEv {
    fn react(self, ctx: &egui::Context, app: &mut App) {
        match self {
            Self::Changelog => app.add_popup(ChangelogPopup),
            Self::Info => app.add_popup(InfoPopup),
            Self::Licenses => app.add_popup(LicensesPopup::default()),
            Self::Manual => app.add_popup(ManualPopup),
            Self::Quit { confirm } => {
                if confirm {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    app.add_popup(QuitPopup);
                }
            }
        }
    }
}
