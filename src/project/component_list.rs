use std::{cmp::Ordering, sync::Arc};

use itertools::Itertools;
use rand::distr::{Alphanumeric, SampleString};

use crate::project::{pla3::PlaComponent, skin::Skin};

#[derive(Debug, Clone, Default)]
pub struct ComponentList(Vec<ComponentListItem>);
#[derive(Debug, Clone)]
struct ComponentListItem {
    pub value: Arc<PlaComponent>,
    pub order: usize,
}

impl PartialEq for ComponentListItem {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.value, &other.value)
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
            .then_with(|| self.value.namespace.cmp(&other.value.namespace))
            .then_with(|| self.value.id.cmp(&other.value.id))
    }
}

impl ComponentListItem {
    fn from_component(item: PlaComponent, skin: &Skin) -> Self {
        Self {
            order: skin.order[item.ty.name()],
            value: Arc::new(item),
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
    pub fn iter(&self) -> impl Iterator<Item = &Arc<PlaComponent>> {
        self.0.iter().map(|a| &a.value)
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Arc<PlaComponent>> {
        self.0.iter_mut().map(|a| &mut a.value)
    }
    pub fn reorder(&mut self, item: Arc<PlaComponent>) {
        let (i_old, item) = self
            .0
            .iter()
            .find_position(|a| Arc::ptr_eq(&a.value, &item))
            .unwrap();
        let i_new = self.insert_position(item);
        match i_old.cmp(&i_new) {
            Ordering::Less => self.0[i_old..i_new].rotate_left(1),
            Ordering::Greater => self.0[i_new..=i_old].rotate_right(1),
            Ordering::Equal => {}
        }
    }
    pub fn for_each<T, F: FnMut(&mut Arc<PlaComponent>) -> T>(
        &mut self,
        skin: &Skin,
        mut f: F,
    ) -> Vec<T> {
        let mut reorders = Vec::new();
        let out = self
            .0
            .iter_mut()
            .map(|ComponentListItem { value, order }| {
                let old_component_type = Arc::clone(&value.ty);
                let old_layer = value.layer;
                let out = f(value);

                #[expect(clippy::float_cmp)]
                if !Arc::ptr_eq(&old_component_type, &value.ty) || old_layer != value.layer {
                    *order = skin.order[value.ty.name()];
                    reorders.push(Arc::clone(value));
                }

                out
            })
            .collect();
        for component in reorders {
            self.reorder(component);
        }

        out
    }
    pub fn remove_namespace(&mut self, namespace: &str) {
        self.0.retain(|a| a.value.namespace != namespace);
    }
    pub fn get_new_id(&self, namespace: &str) -> String {
        let id = Alphanumeric.sample_string(&mut rand::rng(), 16);
        if self
            .0
            .iter()
            .find(|a| a.value.namespace == namespace && a.value.id == id)
            .is_some()
        {
            return self.get_new_id(namespace);
        }
        id
    }
}
