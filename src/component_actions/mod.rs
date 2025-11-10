use std::{collections::HashMap, sync::Arc};

use tracing::info;

use crate::{
    App,
    project::{
        event::Event,
        pla3::{FullId, PlaComponent},
    },
};

pub mod create;
pub mod paint;
pub mod select;

#[derive(Clone, PartialEq, Debug)]
pub enum ComponentEv {
    Create(Vec<PlaComponent>),
    Delete(Vec<PlaComponent>),
    ChangeField {
        before: Vec<PlaComponent>,
        after: Vec<PlaComponent>,
        label: &'static str,
    },
}

impl Event for ComponentEv {
    fn run(&self, _ctx: &egui::Context, app: &mut App) -> bool {
        match self {
            Self::Create(components) => {
                for component in components {
                    app.project
                        .components
                        .insert(app.project.skin().unwrap(), component.clone());
                }
                true
            }
            Self::Delete(components) => app.project.components.remove_multiple(components),
            Self::ChangeField { before, after, .. } => {
                for (before, after) in before.iter().zip(after) {
                    let Some(comp) = app.project.components.iter_mut().find(|a| **a == *before)
                    else {
                        return false;
                    };
                    *comp = after.clone();
                }
                true
            }
        }
    }
    fn undo(&self, ctx: &egui::Context, app: &mut App) -> bool {
        match self {
            Self::Create(component) => Self::Delete(component.clone()),
            Self::Delete(component) => Self::Create(component.clone()),
            Self::ChangeField {
                before,
                after,
                label,
            } => Self::ChangeField {
                before: after.clone(),
                after: before.clone(),
                label: *label,
            },
        }
        .run(ctx, app)
    }
}
