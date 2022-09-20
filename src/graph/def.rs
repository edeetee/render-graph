use std::rc::Rc;

use egui::TextureId;
use glium::{texture::SrgbTexture2d, Texture2d};
use strum::{EnumIter, IntoStaticStr, Display};
use isf::{Isf, Input, InputType};

use super::{connection_types::{DEFAULT_TEXTURE2D_INPUT, NodeOutputDef, NodeInputDef}, isf::IsfFile};

pub struct NodeData {
    pub template: NodeTypes,
    pub result: Option<TextureId>, // pub texture_cache: Option<ShaderData>
}

#[derive(PartialEq, Eq, Display, Clone, Copy)]
pub enum NodeConnectionTypes {
    // FrameBuffer,
    Texture2D,
    Vec2,
    Float,
}

impl From<InputType> for NodeValueTypes {
    fn from(ty: InputType) -> Self {
        Self(ty)
    }
}

pub struct NodeValueTypes(pub InputType);

// #[derive(Debug)]
// pub enum NodeValueTypes {
//     // Vec2([f32; 2]),
//     // Float(f32),
//     // Bool(bool),
//     Isf(InputType),
//     None,
// }

pub enum ComputedNodeInput<'a> {
    Vec2(&'a [f32; 2]),
    Float(&'a f32),
    Bool(&'a bool),
    Texture(Rc<Texture2d>),
}

// impl From<[f32; 2]> for NodeValueTypes {
//     fn from([x, y]: [f32; 2]) -> Self {
//         Self::Vec2([x, y])
//     }
// }

// impl From<f32> for NodeValueTypes {
//     fn from(val: f32) -> Self {
//         Self::Float(val)
//     }
// }

// impl From<bool> for NodeValueTypes {
//     fn from(val: bool) -> Self {
//         Self::Bool(val)
//     }
// }

#[derive(Clone, PartialEq)]
pub enum NodeTypes {
    Instances,
    // Feedback,
    // Sdf,
    // Uv,
    Output,
    Isf {
        file: IsfFile,
        isf: Isf,
    }
}

#[derive(Debug, Clone)]
pub enum GraphResponse {}

#[derive(Default)]
pub struct GraphState {}
