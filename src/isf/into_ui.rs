use core::convert::From;
use core::default::Default;
use egui::Rgba;
use isf::{Input, InputType, InputValues};

use crate::common::node_connections::InputDef;
use crate::common::def::{ConnectionType, UiValue, RangedData};

impl From<&InputType> for ConnectionType {
    fn from(ty: &InputType) -> Self {
        match ty {
            InputType::Image => ConnectionType::Texture2D,
            // InputType::Float(_) => Ok(NodeConnectionTypes::None),
            // InputType::Point2d(_) => NodeConnectionTypes::Texture2D,
            _ => ConnectionType::None
        }
    }
}

impl <T: Default + Copy> From<&InputValues<T>> for RangedData<T> {
    fn from(value: &InputValues<T>) -> Self {
        Self {
            value: value.identity.or(value.default).unwrap_or_default(),
            min: value.min,
            max: value.max,
            default: value.default
        }
    }
}

fn slice_from_vec(input: &Vec<f32>) -> [f32; 4] {
    let mut slice = [0.0; 4];
    for (i, v) in input.iter().enumerate() {
        slice[i] = *v;
    }
    slice
}

fn rgba_from_vec(input: &Vec<f32>) -> Rgba {
    let slice = slice_from_vec(input);
    Rgba::from_rgba_premultiplied(slice[0], slice[1], slice[2], slice[3])
}

impl From<&InputType> for UiValue {
    fn from(ty: &InputType) -> Self {
        match ty {
            InputType::Float(v) => UiValue::Float(v.into()),
            InputType::Color(v) => {

                let default = vec![];

                let data = RangedData{
                    value: rgba_from_vec(v.default.as_ref().unwrap_or(&default)),
                    min: v.min.as_ref().map(rgba_from_vec),
                    max: v.max.as_ref().map(rgba_from_vec),
                    default: v.default.as_ref().map(rgba_from_vec)
                };

                UiValue::Color(data)
            },
            InputType::Point2d(v) => UiValue::Vec2(v.into()),
            InputType::Bool(v) => UiValue::Bool(v.default.unwrap_or_default().into()),
            InputType::Long(v) => UiValue::Long(
                RangedData {
                    value: v.default.unwrap_or_default(),
                    min: v.min.or_else(|| v.values.iter().min().copied()),
                    max: v.max.or_else(|| v.values.iter().max().copied()),
                    default: v.default
                }
            ),
            
            InputType::Event => UiValue::Bool(false.into()),

            InputType::Image | InputType::Audio(_) | InputType::AudioFft(_) => UiValue::None,
        }
    }
}

impl From<&Input> for InputDef {
    fn from(input: &Input) -> Self {
        let ty = (&input.ty).into();
        let value = (&input.ty).into();

        Self {
            name: input.name.clone(),
            ty,
            value,
        }
    }
}
