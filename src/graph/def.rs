use std::rc::Rc;

use egui::TextureId;
use glium::{texture::SrgbTexture2d, Texture2d};
use strum::{EnumIter, IntoStaticStr};

use super::connection_types::{DEFAULT_TEXTURE2D_INPUT, NodeOutputDef, NodeInputDef};

pub struct NodeData {
    pub template: NodeTypes,
    pub result: Option<TextureId>, // pub texture_cache: Option<ShaderData>
}

#[derive(PartialEq, Eq, IntoStaticStr, Clone, Copy)]
pub enum NodeConnectionTypes {
    // FrameBuffer,
    Texture2D,
    Vec2,
    Float,
}

#[derive(Debug)]
pub enum NodeValueTypes {
    Vec2([f32; 2]),
    Float(f32),
    Bool(bool),
    None,
}

pub enum ComputedNodeInput<'a> {
    Vec2(&'a [f32; 2]),
    Float(&'a f32),
    Bool(&'a bool),
    Texture(Rc<Texture2d>),
}

impl From<[f32; 2]> for NodeValueTypes {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::Vec2([x, y])
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

#[derive(Clone, Copy, IntoStaticStr, EnumIter, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeTypes {
    Instances,
    Feedback,
    Sdf,
    Uv,
    Output,
}

impl NodeTypes {
    pub fn get_input_types(&self) -> Vec<NodeInputDef> {
        match self {
            NodeTypes::Uv => vec![
                ("scale", [1., 1.]).into(),
                ("centered", false).into(),
            ],
            NodeTypes::Sdf => vec![
                NodeInputDef::new_texture("uv"),
            ],
            _ => vec![DEFAULT_TEXTURE2D_INPUT],
        }
    }

    pub fn get_output_types(&self) -> Vec<NodeOutputDef> {
        match self {
            NodeTypes::Output => vec![],
            _ => vec![NodeConnectionTypes::Texture2D.into()],
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphResponse {}

#[derive(Default)]
pub struct GraphState {}
