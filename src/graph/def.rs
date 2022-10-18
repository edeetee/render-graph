



use std::{rc::{Weak}, cell::RefCell, path::PathBuf};

use egui::{Rgba};
use egui_node_graph::{GraphEditorState, UserResponseTrait};
use glam::{Mat4, Vec3, EulerRot};
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

///just set value and default
impl <T> From<T> for RangedData<T>
    where T: Clone
{
    ///Set default and value
    fn from(value: T) -> Self {
        Self {
            value: value.clone(),
            min: None,
            max: None,
            default: Some(value)
        }
    }
}

// impl From<T>

impl <T: PartialEq> PartialEq for RangedData<T>{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Debug, PartialEq)]
pub struct Mat4UiData {
    pub mat: Mat4,
    pub rotation: (f32, f32, f32),
    pub scale: f32,
    pub translation: Vec3
}

const EULER_ORDER: EulerRot = EulerRot::ZXY;

impl From<Mat4> for Mat4UiData {
    fn from(value: Mat4) -> Self {
        let decomposed = value.to_scale_rotation_translation();
        Self {
            scale: decomposed.0.length_squared()/3.0,
            rotation: decomposed.1.to_euler(EULER_ORDER),
            translation: decomposed.2,
            mat: value,
        }
    }
}

impl Mat4UiData {
    pub fn new_view() -> Self {
        let mut new = Self {
            translation: Vec3::new(0.0, 0.0, -5.0),
            mat: Mat4::IDENTITY,
            scale: 1.0,
            rotation: Default::default()
        };

        new.update_mat();

        new
    }

    pub fn update_mat(&mut self) {
        self.mat = Mat4::IDENTITY
            * Mat4::from_euler(EULER_ORDER, self.rotation.2, self.rotation.0, self.rotation.1)
            * Mat4::from_translation(self.translation)
            * Mat4::from_scale(Vec3::new(self.scale, self.scale, self.scale))
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
    Mat4(Mat4UiData),
    None,
}

impl From<&str> for UiValue {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string().into())
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
            UiValue::Mat4(v) => Some(UniformValue::Mat4(v.mat.to_cols_array_2d())),

            UiValue::Text(_) | UiValue::Path(_) | UiValue::None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphResponse;
impl UserResponseTrait for GraphResponse {}

pub struct GraphState;

pub(crate) type EditorState =
    GraphEditorState<NodeData, ConnectionType, UiValue, NodeType, GraphState>;

pub type Graph = egui_node_graph::graph::Graph<NodeData, ConnectionType, UiValue>;