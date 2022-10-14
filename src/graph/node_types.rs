


use std::{rc::Weak, fmt::Display};

use egui_node_graph::{NodeTemplateTrait, Graph, NodeId, NodeTemplateIter};
use super::{def::*, conection_def::{InputDef, OutputDef}};
use crate::isf::meta::{parse_isf_shaders, IsfInfo, default_isf_path};

#[derive(Clone, PartialEq, Debug)]
pub enum NodeType {
    Instances,
    SpoutIn,
    SpoutOut,
    Output,
    Isf {
        info: IsfInfo
    }
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

impl NodeType {
    pub fn get_name(&self) -> &str {
        match self {
            NodeType::SpoutIn => "SpoutIn",
            NodeType::SpoutOut => "SpoutOut",
            NodeType::Instances => "Instances",
            NodeType::Output => "Output",
            NodeType::Isf{info} => info.name.as_str()
        }
    }

    pub fn get_builtin() -> Vec<NodeType> {
        // let path = default_isf_path();
        // let shaders = parse_isf_shaders(&path)
        //     .map(|info| NodeType::Isf{info});

        let mut types = vec![
            // NodeTypes::Instances,
            // NodeTypes::Output,
            NodeType::SpoutOut
        ];
        // types.extend(shaders);

        types
    }

    pub fn get_input_types(&self) -> Vec<InputDef> {
        match self {
            NodeType::Isf { info } => {
                info.def.inputs.iter().map(InputDef::from).collect()
            }
            NodeType::SpoutOut => vec![
                ("name", "RustSpout").into(),
                InputDef::texture("texture"),
            ],
            _ => vec![InputDef::texture("Texture2D")],
        }
    }

    pub fn get_output_types(&self) -> Vec<OutputDef> {
        match self {
            NodeType::Output => vec![],
            NodeType::SpoutOut => vec![],
            _ => vec![ConnectionType::Texture2D.into()],
        }
    }
}
pub struct AllNodeTypes;
impl NodeTemplateIter for AllNodeTypes {
    type Item = NodeType;

    fn all_kinds(&self) -> Vec<Self::Item> {
        NodeType::get_builtin()
    }
}

impl NodeTemplateTrait for NodeType {
    type NodeData = NodeData;
    type DataType = ConnectionType;
    type ValueType = UiValue;

    fn node_finder_label(&self) -> &str {
        self.get_name()
    }

    fn node_graph_label(&self) -> String {
        self.node_finder_label().into()
    }

    fn user_data(&self) -> Self::NodeData {
        NodeData { template: self.clone(), texture: Weak::default() }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        node_id: NodeId
    ) {
        for input in self.get_input_types() {
            let connection = input.ty != ConnectionType::None;
            let value = input.value != UiValue::None;

            let kind = match (connection, value) {
                (true, true) => egui_node_graph::InputParamKind::ConnectionOrConstant,
                (true, false) => egui_node_graph::InputParamKind::ConnectionOnly,
                (false, true) => egui_node_graph::InputParamKind::ConstantOnly,
                (false, false) => continue
            };

            graph.add_input_param(node_id, input.name, input.ty, input.value, kind, true);
        }

        for output in self.get_output_types() {
            graph.add_output_param(node_id, output.name, output.ty);
        }
    }
}