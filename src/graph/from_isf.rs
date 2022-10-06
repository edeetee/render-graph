use core::convert::From;
use core::default::Default;
use core::option::Option::Some;
use egui::Rgba;
use isf::{Input, InputType};
use super::conection_def::NodeInputDef;
use super::def::{NodeConnectionTypes, NodeValueTypes};

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

impl From<&InputType> for NodeValueTypes {
    fn from(ty: &InputType) -> Self {
        match ty {
            InputType::Float(v) => NodeValueTypes::Float(v.default.unwrap_or_default()),
            InputType::Color(v) => {
                let mut slice: [f32; 4] = Default::default();
                if let Some(default) = &v.default{
                    for (from, to) in default.iter().zip(&mut slice){
                        *to = *from;
                    }
                }
                let rgba = Rgba::from_rgba_premultiplied(slice[0], slice[1], slice[2], slice[3]);
                NodeValueTypes::Color(rgba)
            },
            InputType::Point2d(v) => NodeValueTypes::Vec2(v.default.unwrap_or_default()),
            InputType::Bool(v) => NodeValueTypes::Bool(v.default.unwrap_or_default()),
            _ => NodeValueTypes::None
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
