


use std::rc::Rc;

use glium::{uniforms::{Uniforms, UniformValue, AsUniformValue}, Texture2d};
use rand::distributions::Uniform;


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

pub struct ComputedInputs<'a> {
    vec: Vec<(&'a str, UniformValue<'a>)>,
}

impl ComputedInputs<'_> {
    pub fn first_texture(&self) -> Option<&Texture2d> {
        self.vec.iter().filter_map(|(_,tex)| {
            match *tex {
                UniformValue::Texture2d(tex, _) => Some(tex),
                _ => None,
            }
        }).next()
    }
}

impl<'a> Uniforms for &ComputedInputs<'a>{
    fn visit_values<'b, F: FnMut(&str, UniformValue<'b>)>(&'b self, mut output: F) {
        for (name, input) in self.vec.iter() {
            output(name, *input);
        }
    }
}

impl<'a> FromIterator<(&'a str, UniformValue<'a>)> for ComputedInputs<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (&'a str, UniformValue<'a>)>,
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