use std::rc::Rc;

use egui::{TextureId, Rgba};
use egui_node_graph::GraphEditorState;
use glium::{Texture2d, uniforms::AsUniformValue};
use strum::{Display};
use isf::{Isf};

use super::{ isf::IsfPathInfo, node_types::NodeTypes};

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

#[derive(Debug, PartialEq)]
pub enum NodeValueTypes {
    Vec2([f32; 2]),
    Float(f32),
    Bool(bool),
    Vec4([f32; 4]),
    Color(Rgba),
    Text(String),
    None,
}

impl From<&str> for NodeValueTypes {
    fn from(s: &str) -> Self {
        Self::Text(s.into())
    }
}

impl NodeValueTypes {
    pub fn as_shader_input(&self) -> Option<ComputedShaderInput> {
        match *self {
            NodeValueTypes::Vec2(v) => Some(v.into()),
            NodeValueTypes::Float(v) => Some(v.into()),
            NodeValueTypes::Bool(v) => Some(v.into()),
            NodeValueTypes::Vec4(v) => Some(v.into()),
            NodeValueTypes::Color(v) => Some(v.to_array().into()),
            _ => None,
        }
    }
}

impl From<[f32; 4]> for ComputedShaderInput {
    fn from(v: [f32; 4]) -> Self {
        Self::Vec4(v)
    }
}

impl From<[f32; 2]> for ComputedShaderInput {
    fn from(v: [f32; 2]) -> Self {
        Self::Vec2(v)
    }
}

impl  From<f32> for ComputedShaderInput {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}

impl  From<bool> for ComputedShaderInput {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<Rc<Texture2d>> for ComputedShaderInput {
    fn from(v: Rc<Texture2d>) -> Self {
        Self::Texture(v)
    }
}

pub enum ComputedShaderInput {
    Vec2([f32; 2]),
    Vec4([f32; 4]),
    Float(f32),
    Bool(bool),
    Texture(Rc<Texture2d>),
}

impl AsUniformValue for ComputedShaderInput {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        match self {
            ComputedShaderInput::Vec2(x) => x.as_uniform_value(),
            ComputedShaderInput::Vec4(x) => x.as_uniform_value(),
            ComputedShaderInput::Float(x) => x.as_uniform_value(),
            ComputedShaderInput::Bool(x) => x.as_uniform_value(),
            ComputedShaderInput::Texture(x) => x.as_uniform_value(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphResponse {}

#[derive(Default)]
pub struct GraphState {}

pub(crate) type EditorState =
    GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;