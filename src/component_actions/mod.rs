use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    sync::Arc,
};

use itertools::Itertools;
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

impl Display for ComponentEv {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentEv::Create(components) => write!(
                f,
                "Create components {}",
                components
                    .iter()
                    .map(|a| format!("{}", a.full_id))
                    .join(", ")
            ),
            ComponentEv::Delete(components) => write!(
                f,
                "Create components {}",
                components
                    .iter()
                    .map(|a| format!("{}", a.full_id))
                    .join(", ")
            ),
            ComponentEv::ChangeField { after, label, .. } => write!(
                f,
                "Change component data ({label}) of {}",
                after.iter().map(|a| format!("{}", a.full_id)).join(", ")
            ),
        }
    }
}
