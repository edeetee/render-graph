

use serde::{Serialize, Deserialize};

use super::def::{ConnectionType, UiValue};

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
}

impl <S: Into<String>, V: Into<UiValue>> From<(S, V)> for InputDef {
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

impl <S: Into<String>> From<(S, ConnectionType)> for OutputDef {
    fn from((name, ty): (S, ConnectionType)) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

