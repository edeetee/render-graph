use core::convert::From;
use core::default::Default;
use egui::Rgba;
use isf::{Input, InputType, InputValues};
use super::conection_def::NodeInputDef;
use super::def::{NodeConnectionTypes, NodeValueTypes, NodeValueData};

impl From<&InputType> for NodeConnectionTypes {
    fn from(ty: &InputType) -> Self {
        match ty {
            InputType::Image => NodeConnectionTypes::Texture2D,
            // InputType::Float(_) => Ok(NodeConnectionTypes::None),
            // InputType::Point2d(_) => NodeConnectionTypes::Texture2D,
            _ => NodeConnectionTypes::None
        }
    }
}

impl <T: Default + Copy> From<&InputValues<T>> for NodeValueData<T> {
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

impl From<&InputType> for NodeValueTypes {
    fn from(ty: &InputType) -> Self {
        match ty {
            InputType::Float(v) => NodeValueTypes::Float(v.into()),
            InputType::Color(v) => {

                let default = vec![];

                let data = NodeValueData{
                    value: rgba_from_vec(v.default.as_ref().unwrap_or(&default)),
                    min: v.min.as_ref().map(rgba_from_vec),
                    max: v.max.as_ref().map(rgba_from_vec),
                    default: v.default.as_ref().map(rgba_from_vec)
                };

                NodeValueTypes::Color(data)
            },
            InputType::Point2d(v) => NodeValueTypes::Vec2(v.into()),
            InputType::Bool(v) => NodeValueTypes::Bool(NodeValueData::new_default(v.default.unwrap_or_default())),
            InputType::Long(v) => NodeValueTypes::Long((&v.input_values).into()),
            
            InputType::Event => NodeValueTypes::Bool(NodeValueData::new_default(false)),

            InputType::Image | InputType::Audio(_) | InputType::AudioFft(_) => NodeValueTypes::None,
        }
    }
}

impl From<&Input> for NodeInputDef {
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
