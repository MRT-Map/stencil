use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use crate::{Error, FullId, PlaComponent, PlaNode, PlaNodeType, PlaNodeTypeBezier, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pla2Component<T: PlaNodeType> {
    namespace: String,
    id: String,
    display_name: String,
    description: String,
    r#type: String,
    layer: NotNan<f32>,
    nodes: Vec<T>,
    tags: Vec<String>,
    attrs: Option<BTreeMap<String, toml::Value>>,
}

impl<T: PlaNodeType> Pla2Component<T> {
    pub fn to_pla3<S: ?Sized, GT: Fn(&str) -> Option<Arc<S>>>(
        self,
        get_type: GT,
    ) -> Result<PlaComponent<S, T>> {
        Ok(PlaComponent {
            full_id: FullId::new(self.namespace, self.id),
            ty: if let Some(ty) = get_type(&self.r#type) {
                ty
            } else if self.nodes.len() == 1 {
                get_type("simplePoint").ok_or_else(|| Error::MissingType("simplePoint".into()))?
            } else {
                get_type("simpleLine").ok_or_else(|| Error::MissingType("simpleLine".into()))?
            },
            display_name: self.display_name,
            layer: self.layer,
            nodes: self
                .nodes
                .into_iter()
                .map(|n| PlaNode::Line {
                    coord: n,
                    label: None,
                })
                .collect(),
            misc: {
                let mut misc = self.attrs.unwrap_or_default();
                if !self.description.is_empty() {
                    misc.insert("description".into(), self.description.into());
                }
                for tag in self.tags {
                    if misc.contains_key(&tag) {
                        return Err(Error::KeyAlreadyExistsForTag(tag));
                    }
                    misc.insert(tag, true.into());
                }
                misc
            },
        })
    }
    pub fn as_pla3<S: ?Sized, GT: Fn(&str) -> Option<Arc<S>>>(
        &self,
        get_type: GT,
    ) -> Result<PlaComponent<S, T>> {
        self.clone().to_pla3(get_type)
    }
}
impl<S: ?Sized, T: PlaNodeTypeBezier> PlaComponent<S, T> {
    pub fn to_pla2<TS: Fn(&S) -> V, V: Into<String>, Tolerance: Into<Option<f32>> + Copy>(
        mut self,
        format_ty: TS,
        tolerance: Tolerance,
    ) -> Result<Pla2Component<T>> {
        Ok(Pla2Component {
            namespace: self.full_id.namespace,
            id: self.full_id.id,
            display_name: self.display_name,
            description: self
                .misc
                .remove("description")
                .map_or_else(String::new, |description| description.to_string()),
            r#type: format_ty(&*self.ty).into(),
            layer: self.layer,
            nodes: self.nodes.outline(tolerance),
            tags: {
                let mut tags = Vec::new();
                self.misc.retain(|k, v| {
                    if v.as_bool() != Some(true) {
                        return true;
                    }
                    tags.push(k.to_owned());
                    false
                });
                tags
            },
            attrs: if self.misc.is_empty() {
                None
            } else {
                Some(self.misc)
            },
        })
    }
    pub fn as_pla2<TS: Fn(&S) -> V, V: Into<String>, Tolerance: Into<Option<f32>> + Copy>(
        &self,
        format_ty: TS,
        tolerance: Tolerance,
    ) -> Result<Pla2Component<T>> {
        self.clone().to_pla2(format_ty, tolerance)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pla2File<T: PlaNodeType> {
    namespace: String,
    components: Vec<Pla2Component<T>>,
}

impl<T: PlaNodeType> Pla2File<T> {
    #[must_use]
    pub fn json_path(&self, root: &Path) -> PathBuf {
        root.join(self.json_file_name())
    }
    #[must_use]
    pub fn json_file_name(&self) -> String {
        format!("{}.pla2.json", self.namespace)
    }

    #[must_use]
    pub fn msgpack_path(&self, root: &Path) -> PathBuf {
        root.join(self.msgpack_file_name())
    }
    #[must_use]
    pub fn msgpack_file_name(&self) -> String {
        format!("{}.pla2.msgpack", self.namespace)
    }
}
impl<T: PlaNodeType + Serialize> Pla2File<T> {
    pub fn as_json_string(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self)
    }
    pub fn as_json_bytes(&self) -> serde_json::error::Result<Vec<u8>> {
        serde_json::to_vec(self)
    }
    pub fn as_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec_named(self)
    }
}
impl<'de, T: PlaNodeType + Deserialize<'de>> Pla2File<T> {
    pub fn from_json_string(input: &'de str) -> serde_json::error::Result<Self> {
        serde_json::from_str(input)
    }
    pub fn from_json_bytes(input: &'de [u8]) -> serde_json::error::Result<Self> {
        serde_json::from_slice(input)
    }
    pub fn from_msgpack(input: &'de [u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(input)
    }
}
