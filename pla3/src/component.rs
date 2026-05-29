use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use ordered_float::NotNan;

use crate::{
    error::{Error, InvalidLabelError, InvalidLayerError, Result},
    node::PlaNode,
    node_type::{PlaNodeType, PlaNodeTypeGet, PlaNodeTypeNew},
    node_vec::PlaNodeVec,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullId {
    pub namespace: String,
    pub id: String,
}

impl Display for FullId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.namespace, self.id)?;
        Ok(())
    }
}

impl FullId {
    #[must_use]
    pub const fn new(namespace: String, id: String) -> Self {
        Self { namespace, id }
    }
}

#[derive(Debug, PartialEq)]
pub struct PlaComponent<S: ?Sized, T: PlaNodeType> {
    pub full_id: FullId,
    pub ty: Arc<S>,
    pub display_name: String,
    pub layer: NotNan<f32>,
    pub nodes: PlaNodeVec<T>,
    pub misc: BTreeMap<String, toml::Value>,
}

impl<S: ?Sized, T: PlaNodeType> Clone for PlaComponent<S, T> {
    fn clone(&self) -> Self {
        Self {
            full_id: self.full_id.clone(),
            ty: Arc::clone(&self.ty),
            display_name: self.display_name.clone(),
            layer: self.layer,
            nodes: self.nodes.clone(),
            misc: self.misc.clone(),
        }
    }
}

impl<S: ?Sized, T: PlaNodeType> Display for PlaComponent<S, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_id)?;
        if !self.display_name.is_empty() {
            write!(f, " ({})", self.display_name)?;
        }
        Ok(())
    }
}

impl<S: ?Sized, T: PlaNodeType> PlaComponent<S, T> {
    #[must_use]
    pub fn path(&self, root: &Path) -> PathBuf {
        root.join(&*self.full_id.namespace)
            .join(format!("{}.pla3", self.full_id.id))
    }
}

impl<S: ?Sized, T: PlaNodeTypeNew> PlaComponent<S, T>
where
    <T::C as FromStr>::Err: 'static,
{
    fn get_coord(split: &[&str], i: usize) -> Result<T> {
        let (x, y) = (split[i], split[i + 1]);
        let (x, y) = (
            x.parse()
                .map_err(|e| Error::InvalidCoordinate(x.to_owned(), Box::new(e)))?,
            y.parse()
                .map_err(|e| Error::InvalidCoordinate(y.to_owned(), Box::new(e)))?,
        );
        Ok(PlaNodeTypeNew::new(x, y))
    }
    fn get_label(split: &[&str], i: usize) -> Result<Option<u8>> {
        let Some(label) = split.get(i) else {
            return Ok(None);
        };
        let Some(label) = label.strip_suffix("#") else {
            return Err(Error::InvalidLabel(
                label.to_string(),
                InvalidLabelError::MissingPrefix,
            ));
        };
        label
            .parse::<u8>()
            .map_err(|e| Error::InvalidLabel(label.to_owned(), e.into()))
            .map(Some)
    }
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(full_id)))]
    pub fn load_from_string<GT: Fn(&str) -> Option<Arc<S>>>(
        s: &str,
        full_id: FullId,
        get_type: GT,
    ) -> Result<(Self, Option<Error>)> {
        let mut unknown_type_error = None;
        let (nodes_str, attrs_str) = s
            .split_once("\n---\n")
            .ok_or_else(|| Error::MissingSeparator(s.to_owned()))?;

        let nodes = nodes_str
            .split('\n')
            .map(|node_str| {
                let split = node_str.split(' ').collect::<Vec<_>>();
                match split.len() {
                    2 | 3 => Ok(Some(PlaNode::Line {
                        coord: Self::get_coord(&split, 0)?,
                        label: Self::get_label(&split, 2)?,
                    })),
                    4 | 5 => Ok(Some(PlaNode::QuadraticBezier {
                        ctrl: Self::get_coord(&split, 0)?,
                        coord: Self::get_coord(&split, 2)?,
                        label: Self::get_label(&split, 4)?,
                    })),
                    6 | 7 => Ok(Some(PlaNode::CubicBezier {
                        ctrl1: Self::get_coord(&split, 0)?,
                        ctrl2: Self::get_coord(&split, 2)?,
                        coord: Self::get_coord(&split, 4)?,
                        label: Self::get_label(&split, 6)?,
                    })),
                    len => Err(Error::InvalidSplitLength(node_str.to_owned(), len)),
                }
            })
            .filter_map(Result::transpose)
            .collect::<Result<PlaNodeVec<T>>>()?;

        if let Some(node @ PlaNode::Line { .. }) = nodes.first() {
            return Err(Error::FirstNodeIsCurve(format!("{node:?}")));
        }

        let mut display_name = String::new();
        let mut layer = NotNan::<f32>::default();
        let mut ty = if nodes.len() == 1 {
            get_type("simplePoint").ok_or_else(|| Error::MissingType("simplePoint".into()))
        } else {
            get_type("simpleLine").ok_or_else(|| Error::MissingType("simpleLine".into()))
        };
        let mut misc = BTreeMap::<String, toml::Value>::new();
        for (k, v) in toml::from_str::<toml::Table>(attrs_str)? {
            match &*k {
                "display_name" => {
                    v.as_str()
                        .ok_or_else(|| Error::InvalidDisplayName(v.clone()))?
                        .clone_into(&mut display_name);
                }
                "layer" => {
                    let float = if let Some(f) = v.as_float() {
                        f as f32
                    } else if let Some(i) = v.as_integer() {
                        i as f32
                    } else {
                        return Err(Error::InvalidLayer(
                            v,
                            InvalidLayerError::NeitherIntegerNorFloat,
                        ));
                    };
                    layer = NotNan::new(float)
                        .map_err(|e| Error::InvalidLayer(v, InvalidLayerError::IsNaN(e)))?;
                }
                "type" => {
                    let ty_str = v
                        .as_str()
                        .ok_or_else(|| Error::InvalidSkinType(v.clone()))?;
                    if let Some(s) = get_type(ty_str) {
                        ty = Ok(s);
                    } else {
                        unknown_type_error =
                            Some(Error::UnknownType(full_id.clone(), ty_str.into()));
                    }
                }
                _ => {
                    misc.insert(k, v);
                }
            }
        }

        Ok((
            Self {
                full_id,
                ty: ty?,
                display_name,
                layer,
                nodes,
                misc,
            },
            unknown_type_error,
        ))
    }
}

impl<S: ?Sized, T: PlaNodeTypeGet> PlaComponent<S, T> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(self.full_id)))]
    pub fn save_to_string<'a, TS: Fn(&'a S) -> V, V: Into<toml::Value> + 'a>(
        &'a self,
        format_ty: TS,
    ) -> Result<String>
    where
        S: 'a,
    {
        let mut out = String::new();

        for node in &self.nodes {
            match node {
                PlaNode::Line { coord, .. } => write!(out, "{} {}", coord.x(), coord.y())?,
                PlaNode::QuadraticBezier { ctrl, coord, .. } => {
                    write!(out, "{} {} {} {}", ctrl.x(), ctrl.y(), coord.x(), coord.y())?;
                }
                PlaNode::CubicBezier {
                    ctrl1,
                    ctrl2,
                    coord,
                    ..
                } => write!(
                    out,
                    "{} {} {} {} {} {}",
                    ctrl1.x(),
                    ctrl1.y(),
                    ctrl2.x(),
                    ctrl2.y(),
                    coord.x(),
                    coord.y()
                )?,
            }
            if let Some(label) = node.label() {
                writeln!(out, " #{label}")?;
            } else {
                out += "\n";
            }
        }
        out += "---\n";

        let attrs = self
            .misc
            .clone()
            .into_iter()
            .chain([
                ("display_name".into(), self.display_name.clone().into()),
                ("layer".into(), (*self.layer).into()),
                ("type".into(), format_ty(&self.ty).into()),
            ])
            .collect::<toml::Table>();
        out += &toml::to_string_pretty(&attrs)?;
        Ok(out)
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use crate::{FullId, PlaComponent};

    proptest! {
        #[test]
        fn test_loading_no_crash(s in ".*", namespace in ".*", id in ".*") {
            let _ = PlaComponent::<str, (f32, f32)>::load_from_string(&s, FullId::new(namespace, id), |t| Some(t.into()));
        }
    }
}
