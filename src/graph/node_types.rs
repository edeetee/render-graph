


use std::rc::Weak;

use egui_node_graph::{NodeTemplateTrait, Graph, NodeId, NodeTemplateIter};
use super::{def::*, conection_def::{InputDef, OutputDef}};
use crate::isf::meta::{parse_isf_shaders, IsfInfo, default_isf_path};

#[derive(Clone, PartialEq, Debug)]
pub enum NodeTypes {
    Instances,
    SpoutIn,
    SpoutOut,
    Output,
    Isf {
        info: IsfInfo
    }
}

impl <'a> From<&'a NodeTypes> for &'a str {
    fn from(ty: &'a NodeTypes) -> Self {
        match ty {
            NodeTypes::SpoutIn => "SpoutIn",
            NodeTypes::SpoutOut => "SpoutOut",
            NodeTypes::Instances => "Instances",
            NodeTypes::Output => "Output",
            NodeTypes::Isf{info} => info.name.as_str()
        }
    }
}

impl NodeTypes {
    pub fn get_all() -> Vec<NodeTypes> {
        let path = default_isf_path();
        let shaders = parse_isf_shaders(&path)
            .map(|info| NodeTypes::Isf{info});

        let mut types = vec![
            // NodeTypes::Instances,
            // NodeTypes::Output,
            NodeTypes::SpoutOut
        ];
        types.extend(shaders);

        types
    }

    pub fn get_input_types(&self) -> Vec<InputDef> {
        match self {
            NodeTypes::Isf { info } => {
                info.def.inputs.iter().map(InputDef::from).collect()
            }
            NodeTypes::SpoutOut => vec![
                ("name", "RustSpout").into(),
                InputDef::texture("texture"),
            ],
            _ => vec![InputDef::texture("Texture2D")],
        }
    }

    pub fn get_output_types(&self) -> Vec<OutputDef> {
        match self {
            NodeTypes::Output => vec![],
            NodeTypes::SpoutOut => vec![],
            _ => vec![ConnectionType::Texture2D.into()],
        }
    }
}
pub struct AllNodeTypes;
impl NodeTemplateIter for AllNodeTypes {
    type Item = NodeTypes;

    fn all_kinds(&self) -> Vec<Self::Item> {
        NodeTypes::get_all()
    }
}

impl NodeTemplateTrait for NodeTypes {
    type NodeData = NodeData;
    type DataType = ConnectionType;
    type ValueType = UiValue;

    fn node_finder_label(&self) -> &str {
        self.into()
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