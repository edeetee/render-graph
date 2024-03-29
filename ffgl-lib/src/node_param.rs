use egui_node_graph::InputId;
use ffgl::parameters::ParamValue;
use ffgl::Param;
use graph::def::UiValue;
use std::ffi::CStr;
use std::ffi::CString;

#[derive(Debug, Clone)]
pub struct NodeParam {
    // pub(crate) node_id: NodeId,
    pub(crate) param_id: InputId,
    pub(crate) group_name: CString,
    pub(crate) name: CString,
    pub(crate) value: ParamValue,
    pub min: f32,
    pub max: f32,
}

type GraphInput = egui_node_graph::InputParam<graph::connections::ConnectionType, UiValue>;

impl NodeParam {
    pub fn new(input: &GraphInput, input_name: &str, node_name: &str) -> Option<Self> {
        if let Some((value, min, max)) = param_value_from_ui_value(&input.value) {
            Some(NodeParam {
                // node_id: input.node,
                param_id: input.id,
                group_name: CString::new(node_name.as_bytes()).unwrap(),
                name: CString::new(format!(
                    "{}.{input_name}",
                    node_name.chars().take(3).collect::<String>()
                ))
                .unwrap(),
                min: min.unwrap_or(0.0),
                max: max.unwrap_or(1.0),
                value,
            })
        } else {
            None
        }
    }
}

fn param_value_from_ui_value(value: &UiValue) -> Option<(ParamValue, Option<f32>, Option<f32>)> {
    match value {
        UiValue::Float(vf) => Some((ParamValue::Float(vf.value), vf.min, vf.max)),
        UiValue::Mat4(m) => Some((ParamValue::Float(m.scale), None, None)),
        _ => None,
    }
}

impl Param for NodeParam {
    fn name(&self) -> &CStr {
        &self.name
    }

    fn group(&self) -> &CStr {
        &self.group_name
    }

    fn min(&self) -> f32 {
        self.min
    }

    fn max(&self) -> f32 {
        self.max
    }

    fn get(&self) -> ffgl::parameters::ParamValue {
        self.value
    }

    fn set(&mut self, value: ffgl::parameters::ParamValue) {
        self.value = value
    }
}
