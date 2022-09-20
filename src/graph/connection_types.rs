use std::convert::TryInto;

use clap::builder::BoolValueParser;
use isf::InputType;

use super::def::{NodeConnectionTypes, NodeValueTypes, NodeTypes};

impl TryFrom<&NodeValueTypes> for NodeConnectionTypes {
    type Error = ();

    fn try_from(ty: &NodeValueTypes) -> Result<Self, Self::Error> {
        match ty.0 {
            InputType::Image => Ok(Self::Texture2D),
            InputType::Float(_) => Ok(Self::Float),
            InputType::Point2d(_) => Ok(Self::Vec2),
            _ => Err(()),
        }
    }
}

pub struct NodeInputDef {
    pub name: String,
    pub ty: NodeConnectionTypes,
    pub value: NodeValueTypes,
}

pub const DEFAULT_TEXTURE2D_INPUT: NodeInputDef = NodeInputDef {
    name: "Image".into(),
    ty: NodeConnectionTypes::Texture2D,
    value: InputType::Image.into(),
};

impl NodeInputDef {
    pub fn new_texture(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ty: NodeConnectionTypes::Texture2D,
            value: InputType::Image.into(),
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

// impl <T: Into<NodeValueTypes>> From<T> for NodeInputDef {
//     fn from(value: T) -> Self {
//         let value = value.into();
//         let ty: NodeConnectionTypes = (&value).try_into();

//         Self {
//             name: ty.to_string(),
//             ty: ty,
//             value
//         }
//     }
// }

// impl <T: Into<NodeValueTypes>, S: Into<String>> From<(S, T)> for NodeInputDef {
//     fn from((name, value): (S, T)) -> Self {
//         let value = value.into();

//         Self {
//             name: name.into(),
//             ty: (&value).into(),
//             value
//         }
//     }
// }