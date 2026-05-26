use crate::{App, coord::CoordInto, project::skin::SkinType};

pub type PlaNodeWorld = pla3::PlaNode<geo::Coord<i32>>;
pub type PlaNodeScreen = pla3::PlaNode<egui::Pos2>;

pub trait ToScreenExt {
    type Output;
    fn to_screen(&self, app: &App, map_centre: egui::Pos2) -> Self::Output;
}
impl ToScreenExt for PlaNodeWorld {
    type Output = PlaNodeScreen;
    fn to_screen(&self, app: &App, map_centre: egui::Pos2) -> Self::Output {
        let world_to_screen =
            |coord: geo::Coord<i32>| app.map_world_to_screen(map_centre, coord.coord_into());
        match *self {
            Self::Line { coord, label } => PlaNodeScreen::Line {
                coord: world_to_screen(coord),
                label,
            },
            Self::QuadraticBezier { ctrl, coord, label } => PlaNodeScreen::QuadraticBezier {
                ctrl: world_to_screen(ctrl),
                coord: world_to_screen(coord),
                label,
            },
            Self::CubicBezier {
                ctrl1,
                ctrl2,
                coord,
                label,
            } => PlaNodeScreen::CubicBezier {
                ctrl1: world_to_screen(ctrl1),
                ctrl2: world_to_screen(ctrl2),
                coord: world_to_screen(coord),
                label,
            },
        }
    }
}

pub type PlaNodeWorldVec = pla3::PlaNodeVec<geo::Coord<i32>>;
pub type PlaNodeScreenVec = pla3::PlaNodeVec<egui::Pos2>;
impl ToScreenExt for PlaNodeWorldVec {
    type Output = PlaNodeScreenVec;
    fn to_screen(&self, app: &App, map_centre: egui::Pos2) -> Self::Output {
        self.iter().map(|a| a.to_screen(app, map_centre)).collect()
    }
}

pub type PlaComponent = pla3::PlaComponent<SkinType, geo::Coord<i32>>;
