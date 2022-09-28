

use egui::{TextureId, Rgba};
use egui_node_graph::GraphEditorState;
use glium::{uniforms::{AsUniformValue, UniformValue}};
use strum::{Display};


use super::{ node_types::NodeTypes};

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
    pub fn as_shader_input(&self) -> Option<UniformValue<'_>> {
        match self {
            NodeValueTypes::Vec2(v) => Some(v.as_uniform_value()),
            NodeValueTypes::Float(v) => Some(v.as_uniform_value()),
            NodeValueTypes::Bool(v) => Some(v.as_uniform_value()),
            NodeValueTypes::Vec4(v) => Some(v.as_uniform_value()),
            NodeValueTypes::Color(v) => Some(UniformValue::Vec4(v.to_array())),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphResponse {}

#[derive(Default)]
pub struct GraphState {}

pub(crate) type EditorState =
    GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;