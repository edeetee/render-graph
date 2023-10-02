use std::{fmt::Debug, path::PathBuf};

use super::mat4_animator::Mat4Animator;
use glium::uniforms::{AsUniformValue, UniformValue};
use serde::{Deserialize, Serialize};

pub trait Reset {
    fn reset(&mut self);
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
// #[delegate(InnerReset)]
pub enum UiValue {
    Vec2(RangedData<[f32; 2]>),
    Float(RangedData<f32>),
    Long(RangedData<i32>),
    Menu(RangedData<i32>, Vec<(String, i32)>),
    Bool(RangedData<bool>),
    Vec4(RangedData<[f32; 4]>),
    Color(RangedData<[f32; 4]>),
    Text(RangedData<String>, TextStyle),
    Path(Option<PathBuf>),
    Mat4(Mat4Animator),

    #[default]
    None,
}

impl From<&str> for UiValue {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string().into(), TextStyle::Oneline)
    }
}

impl Reset for UiValue {
    fn reset(&mut self) {
        match self {
            UiValue::Vec2(v) => v.reset(),
            UiValue::Float(v) => v.reset(),
            UiValue::Bool(v) => v.reset(),
            UiValue::Vec4(v) => v.reset(),
            UiValue::Color(v) => v.reset(),
            UiValue::Long(v) => v.reset(),
            UiValue::Menu(v, _) => v.reset(),
            UiValue::Mat4(v) => v.reset(),

            UiValue::Text(v, style) => {
                v.reset();
                *style = Default::default()
            }
            UiValue::Path(optional_path) => *optional_path = None,
            UiValue::None => {}
        }
    }
}

impl UiValue {
    pub fn as_shader_input(&self) -> Option<UniformValue<'_>> {
        match self {
            UiValue::Vec2(v) => Some(v.value.as_uniform_value()),
            UiValue::Float(v) => Some(v.value.as_uniform_value()),
            UiValue::Bool(v) => Some(v.value.as_uniform_value()),
            UiValue::Vec4(v) => Some(v.value.as_uniform_value()),
            UiValue::Color(v) => Some(v.value.as_uniform_value()),
            UiValue::Long(v) => Some(v.value.as_uniform_value()),
            UiValue::Menu(v, _) => Some(v.value.as_uniform_value()),
            UiValue::Mat4(v) => Some(UniformValue::Mat4(v.mat.to_cols_array_2d())),

            UiValue::Text(..) | UiValue::Path(_) | UiValue::None => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RangedData<T: Clone + Default> {
    pub value: T,
    pub min: Option<T>,
    pub max: Option<T>,
    pub default: Option<T>,
}

///just set value and default
impl<T> From<T> for RangedData<T>
where
    T: Clone + Default,
{
    ///Set default and value
    fn from(value: T) -> Self {
        Self {
            value: value.clone(),
            min: None,
            max: None,
            default: Some(value),
        }
    }
}

impl<T: Clone + Default> Reset for RangedData<T> {
    fn reset(&mut self) {
        self.value = self.default.clone().unwrap_or_default();
    }
}

impl<T: PartialEq + Clone + Default> PartialEq for RangedData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub enum TextStyle {
    #[default]
    Oneline,
    Multiline,
}
