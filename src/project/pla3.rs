use crate::{App, project::skin::SkinType, utils::coord::CoordInto};

pub type PlaNodeWorld = pla3::PlaNode<geo::Coord<i32>>;
pub type PlaNodeScreen = pla3::PlaNode<egui::Pos2>;

pub trait ToScreenExt {
    type Output;
    fn to_screen(&self, app: &App, map_centre: egui::Pos2) -> Self::Output;
}
impl ToScreenExt for PlaNodeWorld {
    type Output = PlaNodeScreen;
    fn to_screen(&self, app: &App, map_centre: egui::Pos2) -> Self::Output {
        self.map(|coord| app.map_world_to_screen(map_centre, coord.coord_into()))
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
