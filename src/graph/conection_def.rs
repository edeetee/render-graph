use glium::{uniforms::{Uniforms, UniformValue}, Texture2d};

use super::def::{NodeConnectionTypes, NodeValueTypes};

#[derive(Debug)]
pub struct NodeInputDef {
    pub name: String,
    pub ty: NodeConnectionTypes,
    pub value: NodeValueTypes,
}

impl NodeInputDef {
    pub fn texture(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ty: NodeConnectionTypes::Texture2D,
            value: NodeValueTypes::None,
        }
    }
}

impl <S: Into<String>, V: Into<NodeValueTypes>> From<(S, V)> for NodeInputDef {
    fn from((name, val_ty): (S, V)) -> Self {
        Self {
            name: name.into(),
            ty: NodeConnectionTypes::None,
            value: val_ty.into(),
        }
    }
}

pub struct NodeOutputDef {
    pub name: String,
    pub ty: NodeConnectionTypes,
}

impl From<NodeConnectionTypes> for NodeOutputDef {
    fn from(ty: NodeConnectionTypes) -> Self {
        Self {
            name: ty.to_string(),
            ty,
        }
    }
}

impl <S: Into<String>> From<(S, NodeConnectionTypes)> for NodeOutputDef {
    fn from((name, ty): (S, NodeConnectionTypes)) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

