



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
pub enum ConnectionType {
    // FrameBuffer,
    Texture2D,
    None
    // Vec2,
    // Float,
}

#[derive(Debug)]
pub struct ValueData<T> {
    pub value: T,
    pub min: Option<T>,
    pub max: Option<T>,
    pub default: Option<T>
}

impl <T> ValueData<T> {
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
}

impl <T: PartialEq> PartialEq for ValueData<T>{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Debug, PartialEq)]
pub enum UiValue {
    Vec2(ValueData<[f32; 2]>),
    Float(ValueData<f32>),
    Long(ValueData<i32>),
    Bool(ValueData<bool>),
    Vec4(ValueData<[f32; 4]>),
    Color(ValueData<Rgba>),
    Text(ValueData<String>),
    None,
}

impl From<&str> for UiValue {
    fn from(s: &str) -> Self {
        Self::Text(ValueData::new_default(s.into()))
    }
}

impl UiValue {
    pub fn as_shader_input(&self) -> Option<UniformValue<'_>> {
        match self {
            UiValue::Vec2(v) => Some(v.value.as_uniform_value()),
            UiValue::Float(v) => Some(v.value.as_uniform_value()),
            UiValue::Bool(v) => Some(v.value.as_uniform_value()),
            UiValue::Vec4(v) => Some(v.value.as_uniform_value()),
            UiValue::Color(v) => Some(UniformValue::Vec4(v.value.to_array())),
            UiValue::Long(v) => Some(v.value.as_uniform_value()),

            UiValue::Text(_) | UiValue::None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphResponse {}

impl UserResponseTrait for GraphResponse {}

#[derive(Default)]
pub struct GraphState {}

pub(crate) type EditorState =
    GraphEditorState<NodeData, ConnectionType, UiValue, NodeTypes, GraphState>;