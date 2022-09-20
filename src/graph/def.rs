use std::rc::Rc;

use egui::TextureId;
use glium::{texture::SrgbTexture2d, Texture2d, uniforms::AsUniformValue};
use strum::{EnumIter, IntoStaticStr, Display};
use isf::{Isf, Input, InputType};

use super::{ isf::IsfPathInfo};

pub struct NodeData {
    pub template: NodeTypes,
    pub result: Option<TextureId>, // pub texture_cache: Option<ShaderData>
}

#[derive(PartialEq, Eq, Display, Clone, Copy, Debug)]
pub enum NodeConnectionTypes {
    // FrameBuffer,
    Texture2D,
    None
    // Vec2,
    // Float,
}

// impl From<InputType> for NodeValueTypes {
//     fn from(ty: InputType) -> Self {
//         Self(ty)
//     }
// }

// pub struct NodeValueTypes(pub InputType);

#[derive(Debug, PartialEq)]
pub enum NodeValueTypes {
    Vec2([f32; 2]),
    Float(f32),
    Bool(bool),
    Vec4([f32; 4]),
    None,
}

pub enum ComputedNodeInput {
    Vec2([f32; 2]),
    Vec4([f32; 4]),
    Float(f32),
    Bool(bool),
    Texture(Rc<Texture2d>),
}

impl AsUniformValue for ComputedNodeInput {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        match self {
            ComputedNodeInput::Vec2(x) => x.as_uniform_value(),
            ComputedNodeInput::Vec4(x) => x.as_uniform_value(),
            ComputedNodeInput::Float(x) => x.as_uniform_value(),
            ComputedNodeInput::Bool(x) => x.as_uniform_value(),
            ComputedNodeInput::Texture(x) => x.as_uniform_value(),
        }
    }
}

impl From<[f32; 4]> for NodeValueTypes {
    fn from(from: [f32; 4]) -> Self {
        Self::Vec4(from)
    }
}

impl From<[f32; 2]> for NodeValueTypes {
    fn from(from: [f32; 2]) -> Self {
        Self::Vec2(from)
    }
}

impl From<f32> for NodeValueTypes {
    fn from(val: f32) -> Self {
        Self::Float(val)
    }
}

impl From<bool> for NodeValueTypes {
    fn from(val: bool) -> Self {
        Self::Bool(val)
    }
}

#[derive(Clone, PartialEq)]
pub enum NodeTypes {
    Instances,
    // Feedback,
    // Sdf,
    // Uv,
    Output,
    Isf {
        file: IsfPathInfo,
        isf: Isf,
    }
}

#[derive(Debug, Clone)]
pub enum GraphResponse {}

#[derive(Default)]
pub struct GraphState {}
