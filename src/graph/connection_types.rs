use clap::builder::BoolValueParser;

use super::def::{NodeConnectionTypes, NodeValueTypes, NodeTypes};

impl From<&NodeValueTypes> for NodeConnectionTypes {
    fn from(ty: &NodeValueTypes) -> Self {
        match ty {
            NodeValueTypes::None => Self::Texture2D,
            NodeValueTypes::Float(_) => Self::Float,
            NodeValueTypes::Vec2(_) => Self::Vec2,
            NodeValueTypes::Bool(_) => Self::Float
        }
    }
}

pub struct NodeInputDef {
    pub name: &'static str,
    pub ty: NodeConnectionTypes,
    pub value: NodeValueTypes,
}

pub const DEFAULT_TEXTURE2D_INPUT: NodeInputDef = NodeInputDef {
    name: "Image",
    ty: NodeConnectionTypes::Texture2D,
    value: NodeValueTypes::None,
};

impl NodeInputDef {
    pub fn new_texture(name: &'static str) -> Self {
        Self {
            name,
            ty: NodeConnectionTypes::Texture2D,
            value: NodeValueTypes::None,
        }
    }
}

pub struct NodeOutputDef {
    pub name: &'static str,
    pub ty: NodeConnectionTypes,
}

impl From<NodeConnectionTypes> for NodeOutputDef {
    fn from(ty: NodeConnectionTypes) -> Self {
        Self {
            name: ty.into(),
            ty,
        }
    }
}

impl From<(&'static str, NodeConnectionTypes)> for NodeOutputDef {
    fn from((name, ty): (&'static str, NodeConnectionTypes)) -> Self {
        Self {
            name: name,
            ty,
        }
    }
}

impl <T: Into<NodeValueTypes>> From<T> for NodeInputDef {
    fn from(value: T) -> Self {
        let value = value.into();
        let ty: NodeConnectionTypes = (&value).into();

        Self {
            name: ty.into(),
            ty: ty,
            value
        }
    }
}

impl  <T: Into<NodeValueTypes>> From<(&'static str, T)> for NodeInputDef {
    fn from((name, value): (&'static str, T)) -> Self {
        let value = value.into();

        Self {
            name: name,
            ty: (&value).into(),
            value
        }
    }
}