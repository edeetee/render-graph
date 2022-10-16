



use std::{rc::{Weak}, cell::RefCell, path::PathBuf};

use egui::{Rgba};
use egui_node_graph::{GraphEditorState, UserResponseTrait};
use glium::{uniforms::{AsUniformValue, UniformValue}};
use strum::{Display};


use crate::textures::UiTexture;

use super::{ node_types::NodeType};

// pub struct TexInfo {
//     pub id: TextureId,
//     pub size: (u32, u32),
// }

pub struct NodeData {
    pub template: NodeType,
    pub texture: Weak<RefCell<UiTexture>>, // pub texture_cache: Option<ShaderData>
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
pub struct RangedData<T> {
    pub value: T,
    pub min: Option<T>,
    pub max: Option<T>,
    pub default: Option<T>
}

impl <T> RangedData<T> {
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

impl <T: PartialEq> PartialEq for RangedData<T>{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Debug, PartialEq)]
pub enum UiValue {
    Vec2(RangedData<[f32; 2]>),
    Float(RangedData<f32>),
    Long(RangedData<i32>),
    Bool(RangedData<bool>),
    Vec4(RangedData<[f32; 4]>),
    Color(RangedData<Rgba>),
    Text(RangedData<String>),
    Path(Option<PathBuf>),
    None,
}

impl From<&str> for UiValue {
    fn from(s: &str) -> Self {
        Self::Text(RangedData::new_default(s.into()))
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

            UiValue::Text(_) | UiValue::Path(_) | UiValue::None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphResponse {}

impl UserResponseTrait for GraphResponse {}

#[derive(Default)]
pub struct GraphState {}

pub(crate) type EditorState =
    GraphEditorState<NodeData, ConnectionType, UiValue, NodeType, GraphState>;