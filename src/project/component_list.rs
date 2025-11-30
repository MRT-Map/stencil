use std::cmp::Ordering;

use itertools::Itertools;
use rand::distr::{Alphanumeric, SampleString};

use crate::project::{pla3::PlaComponent, skin::Skin};

#[derive(Debug, Clone, Default)]
pub struct ComponentList(Vec<ComponentListItem>);
#[derive(Debug, Clone)]
struct ComponentListItem {
    pub value: PlaComponent,
    pub order: usize,
}

impl PartialEq for ComponentListItem {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl Eq for ComponentListItem {}

impl PartialOrd for ComponentListItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ComponentListItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value
            .layer
            .total_cmp(&other.value.layer)
            .then_with(|| self.order.cmp(&other.order))
            .then_with(|| {
                self.value
                    .full_id
                    .namespace
                    .cmp(&other.value.full_id.namespace)
            })
            .then_with(|| self.value.full_id.id.cmp(&other.value.full_id.id))
    }
}

impl ComponentListItem {
    fn from_component(item: PlaComponent, skin: &Skin) -> Self {
        Self {
            order: skin.order[item.ty.name()],
            value: item,
        }
    }
}

impl ComponentList {
    fn insert_position(&self, item: &ComponentListItem) -> usize {
        fn f(vec: &[ComponentListItem], item: &ComponentListItem, l: usize, h: usize) -> usize {
            let m = l + (h - l) / 2;
            match item.cmp(&vec[m]) {
                Ordering::Equal => m,
                Ordering::Less => {
                    if l == m {
                        l
                    } else {
                        f(vec, item, l, m - 1)
                    }
                }
                Ordering::Greater => {
                    if h == m {
                        h + 1
                    } else {
                        f(vec, item, m + 1, h)
                    }
                }
            }
        }

        if self.0.is_empty() {
            return 0;
        }
        f(&self.0, item, 0, self.0.len() - 1)
    }
    pub fn insert(&mut self, skin: &Skin, item: PlaComponent) {
        let item = ComponentListItem::from_component(item, skin);
        self.0.insert(self.insert_position(&item), item);
    }
    pub fn iter(&self) -> impl Iterator<Item = &PlaComponent> {
        self.0.iter().map(|a| &a.value)
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PlaComponent> {
        self.0.iter_mut().map(|a| &mut a.value)
    }
    pub fn remove_namespace(&mut self, namespace: &str) {
        self.0.retain(|a| a.value.full_id.namespace != namespace);
    }
    pub fn remove_multiple(&mut self, components: &[PlaComponent]) -> bool {
        let positions = self
            .iter()
            .positions(|a| components.contains(a))
            .collect::<Vec<_>>();
        if positions.len() < components.len() {
            return false;
        }
        self.0 = self
            .0
            .drain(..)
            .enumerate()
            .filter(|(i, _)| !positions.contains(i))
            .map(|(_, a)| a)
            .collect();
        true
    }
    pub fn get_new_id(&self, namespace: &str) -> String {
        let id = Alphanumeric.sample_string(&mut rand::rng(), 16);
        if self
            .0
            .iter()
            .any(|a| a.value.full_id.namespace == namespace && a.value.full_id.id == id)
        {
            return self.get_new_id(namespace);
        }
        id
    }
}
