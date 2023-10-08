use crate::GraphState;

use super::def::UiValue;
use egui::color::Hsva;
use egui_node_graph::DataTypeTrait;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(PartialEq, Eq, Display, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ConnectionType {
    Texture2D,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputDef {
    pub name: String,
    pub ty: ConnectionType,
    pub value: UiValue,
}

impl InputDef {
    pub fn texture(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ty: ConnectionType::Texture2D,
            value: UiValue::None,
        }
    }

    pub fn kind(&self) -> Option<egui_node_graph::InputParamKind> {
        let connection = self.ty != ConnectionType::None;
        let value = self.value != UiValue::None;

        match (connection, value) {
            (true, true) => Some(egui_node_graph::InputParamKind::ConnectionOrConstant),
            (true, false) => Some(egui_node_graph::InputParamKind::ConnectionOnly),
            (false, true) => Some(egui_node_graph::InputParamKind::ConstantOnly),
            (false, false) => None,
        }
    }
}

impl<S: Into<String>, V: Into<UiValue>> From<(S, V)> for InputDef {
    fn from((name, val_ty): (S, V)) -> Self {
        Self {
            name: name.into(),
            ty: ConnectionType::None,
            value: val_ty.into(),
        }
    }
}

pub struct OutputDef {
    pub name: String,
    pub ty: ConnectionType,
}

impl From<ConnectionType> for OutputDef {
    fn from(ty: ConnectionType) -> Self {
        Self {
            name: ty.to_string(),
            ty,
        }
    }
}

impl<S: Into<String>> From<(S, ConnectionType)> for OutputDef {
    fn from((name, ty): (S, ConnectionType)) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

impl DataTypeTrait<GraphState> for ConnectionType {
    fn data_type_color(&self, _: &mut GraphState) -> egui::Color32 {
        let hue = match self {
            ConnectionType::Texture2D => 0.7,
            ConnectionType::None => 0.0,
        };

        Hsva::new(hue, 1., 1., 1.).into()
    }

    fn name(&self) -> std::borrow::Cow<str> {
        self.to_string().into()
    }
}
