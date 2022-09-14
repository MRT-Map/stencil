use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;

#[derive(Component)]
pub struct SelectShadow;
#[derive(Bundle)]
pub struct SelectShadowBundle {
    pub _marker: SelectShadow,
    #[bundle]
    pub shape: ShapeBundle,
}
