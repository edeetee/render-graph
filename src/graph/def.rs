

use std::ops::Sub;

use egui::{TextureId, Rgba};
use egui_node_graph::{GraphEditorState, UserResponseTrait};
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

#[derive(Debug)]
pub struct NodeValueData<T> {
    pub value: T,
    pub min: Option<T>,
    pub max: Option<T>,
    pub default: Option<T>
}

impl <T> NodeValueData<T> {
    ///Set default and value
    pub fn new_default(value: T) -> Self
        where T: Clone
    {
        Self {
            value: value.clone(),
            min: None,
            max: None,
            default: Some(value)
        }
    }

    ///return the range between min and max.
    ///If no min and max, returns None
    pub fn range(&self) -> Option<<T as Sub>::Output>
        where T: Sub + Copy
    {
        match self {
            Self { min: Some(min), max: Some(max), .. } => Some(*max - *min),
            _ => None
        }
    }

    // pub fn range(&self) -> Option<T>
    //     where T: 
    // {

    // }
}

fn slice_diff<const A: usize>(a: &[f32; A], b: &[f32; A]) -> [f32; A] {
    let mut out = [f32::default(); A];
    for i in 0..4 {
        out[i] =  (a[i] - b[i]).abs();
    }
    
    out
}

impl <T: PartialEq> PartialEq for NodeValueData<T>{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Debug, PartialEq)]
pub enum NodeValueTypes {
    Vec2(NodeValueData<[f32; 2]>),
    Float(NodeValueData<f32>),
    Long(NodeValueData<i32>),
    Bool(NodeValueData<bool>),
    Vec4(NodeValueData<[f32; 4]>),
    Color(NodeValueData<Rgba>),
    Text(NodeValueData<String>),
    None,
}

impl From<&str> for NodeValueTypes {
    fn from(s: &str) -> Self {
        Self::Text(NodeValueData::new_default(s.into()))
    }
}

impl NodeValueTypes {
    pub fn as_shader_input(&self) -> Option<UniformValue<'_>> {
        match self {
            NodeValueTypes::Vec2(v) => Some(v.value.as_uniform_value()),
            NodeValueTypes::Float(v) => Some(v.value.as_uniform_value()),
            NodeValueTypes::Bool(v) => Some(v.value.as_uniform_value()),
            NodeValueTypes::Vec4(v) => Some(v.value.as_uniform_value()),
            NodeValueTypes::Color(v) => Some(UniformValue::Vec4(v.value.to_array())),
            NodeValueTypes::Long(v) => Some(v.value.as_uniform_value()),

            NodeValueTypes::Text(_) | NodeValueTypes::None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphResponse {}

impl UserResponseTrait for GraphResponse {}

#[derive(Default)]
pub struct GraphState {}

pub(crate) type EditorState =
    GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;