use std::{path::PathBuf, default, ops::RangeInclusive, fmt::Debug};
use egui::{Rgba};
use glam::{Mat4, Vec3, EulerRot};
use glium::{uniforms::{AsUniformValue, UniformValue}};
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use strum::{Display};

#[derive(PartialEq, Eq, Display, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ConnectionType {
    // FrameBuffer,
    Texture2D,
    None
    // Vec2,
    // Float,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

///Transformation data with helper data for human editing
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug,Default, PartialEq, Clone, Serialize, Deserialize)]
pub enum TextStyle {
    #[default]
    Oneline,
    Multiline
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum UiValue {
    Vec2(RangedData<[f32; 2]>),
    Float(RangedData<f32>),
    Long(RangedData<i32>),
    Bool(RangedData<bool>),
    Vec4(RangedData<[f32; 4]>),
    Color(RangedData<Rgba>),
    Text(RangedData<String>, TextStyle),
    Path(Option<PathBuf>),
    Mat4(Mat4UiData),
    #[default]
    None,
}

impl From<&str> for UiValue {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string().into(), TextStyle::Oneline)
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

            UiValue::Text(..) | UiValue::Path(_) | UiValue::None => None,
        }
    }
}