use std::{path::PathBuf, fmt::Debug};
use egui::{Rgba};
use glium::{uniforms::{AsUniformValue, UniformValue}};

use serde::{Serialize, Deserialize};

use super::mat4_ui::Mat4UiData;



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


#[derive(Debug,Default, PartialEq, Clone, Serialize, Deserialize)]
pub enum TextStyle {
    #[default]
    Oneline,
    Multiline
}