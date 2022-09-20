use std::convert::TryInto;

use clap::builder::BoolValueParser;
use glium::uniforms::{Uniforms, UniformValue, AsUniformValue};
use isf::InputType;

use super::def::{NodeConnectionTypes, NodeValueTypes, NodeTypes, ComputedNodeInput};

// impl From<&NodeValueTypes> for NodeConnectionTypes {
//     type Error = ();

//     fn try_from(ty: &NodeValueTypes) -> Result<Self, Self::Error> {
//         match ty {
//             NodeValueTypes::Texture => Ok(Self::Texture2D),
//             // NodeValueTypes::Float(_) => Ok(Self::Float),
//             // NodeValueTypes::Vec2(_) => Ok(Self::Vec2),
//             _ => Err(()),
//         }
//     }
// }

#[derive(Debug)]
pub struct NodeInputDef {
    pub name: String,
    pub ty: NodeConnectionTypes,
    pub value: NodeValueTypes,
}

// pub const DEFAULT_TEXTURE2D_INPUT: NodeInputDef = NodeInputDef {
//     name: "Image".into(),
//     ty: NodeConnectionTypes::Texture2D,
//     value: NodeValueTypes::Texture,
// };

impl NodeInputDef {
    pub fn texture(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ty: NodeConnectionTypes::Texture2D,
            value: NodeValueTypes::None,
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

pub struct ComputedInputs<'a> {
    vec: Vec<(&'a str, ComputedNodeInput)>,
}

impl<'a> Uniforms for &ComputedInputs<'a>{
    fn visit_values<'b, F: FnMut(&str, UniformValue<'b>)>(&'b self, mut output: F) {
        for (name, input) in self.vec.iter() {
            output(name, input.as_uniform_value());
        }
    }
}

impl<'a> FromIterator<(&'a str, ComputedNodeInput)> for ComputedInputs<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (&'a str, ComputedNodeInput)>,
    {
        ComputedInputs {
            vec: iter.into_iter().collect(),
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