use crate::common::def::UiValue;
use egui_node_graph::{InputId, NodeId};
use ffgl::parameters::ParamValue;
use ffgl::Param;
use std::ffi::CStr;
use std::ffi::CString;

#[derive(Debug, Clone)]
pub struct NodeParam {
    pub(crate) node_id: NodeId,
    pub(crate) param_id: InputId,
    pub(crate) group_name: CString,
    pub(crate) name: CString,
    pub(crate) value: ParamValue,
}

type GraphInput = egui_node_graph::InputParam<crate::common::connections::ConnectionType, UiValue>;

impl NodeParam {
    pub fn new(input: &GraphInput, input_name: &str, node_name: &str) -> Option<Self> {
        if let Some(value) = (&input.value).into() {
            Some(NodeParam {
                node_id: input.node,
                param_id: input.id,
                group_name: CString::new(node_name.as_bytes()).unwrap(),
                name: CString::new(format!(
                    "{}.{input_name}",
                    node_name.chars().take(3).collect::<String>()
                ))
                .unwrap(),
                value,
            })
        } else {
            None
        }
    }
}

impl From<&UiValue> for Option<ParamValue> {
    fn from(value: &UiValue) -> Self {
        match value {
            UiValue::Float(vf) => Some(ParamValue::Float(vf.value)),
            UiValue::Mat4(m) => Some(ParamValue::Float(m.scale)),
            _ => None,
        }
    }
}

impl Param for NodeParam {
    fn name(&self) -> &CStr {
        &self.name
    }

    fn group(&self) -> &CStr {
        &self.group_name
    }

    fn get(&self) -> ffgl::parameters::ParamValue {
        self.value
    }

    fn set(&mut self, value: ffgl::parameters::ParamValue) {
        self.value = value
    }
}
