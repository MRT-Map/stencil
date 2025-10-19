pub mod pla3;
pub mod skin;

use serde::{Deserialize, Serialize};

use crate::{map::basemap::Basemap, project::skin::Skin};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Project {
    pub basemap: Basemap,
    pub skin: Skin,
}
