use std::{convert::{TryInto}};

use egui::{color::{Hsva}};
use egui_node_graph::{DataTypeTrait, NodeTemplateTrait, Graph, NodeId, NodeTemplateIter, UserResponseTrait};

use super::{def::*, connection_types::{NodeInputDef, NodeOutputDef}, isf::parse_isf_shaders};

impl <'a> From<&'a NodeTypes> for &'a str {
    fn from(ty: &'a NodeTypes) -> Self {
        match ty {
            NodeTypes::Instances => "Instances",
            NodeTypes::Output => "Output",
            NodeTypes::Isf{file, ..} => file.name.as_str()
        }
    }
}

impl NodeTypes {
    pub fn get_all() -> Vec<NodeTypes> {
        let shaders = parse_isf_shaders()
            .map(|(file, isf)| NodeTypes::Isf{file, isf});

        let mut types = vec![
            NodeTypes::Instances,
            NodeTypes::Output,
        ];
        types.extend(shaders);

        types
    }

    pub fn get_input_types(&self) -> Vec<NodeInputDef> {
        match self {
            // NodeTypes::Uv => vec![
            //     ("scale", [1., 1.]).into(),
            //     ("centered", false).into(),
            // ],
            // NodeTypes::Sdf => vec![
            //     NodeInputDef::new_texture("uv"),
            // ],
            NodeTypes::Isf { isf, .. } => {
                isf.inputs.iter().map(NodeInputDef::from).collect()
            }
            _ => vec![NodeInputDef::texture("Texture2D")],
        }
    }

    pub fn get_output_types(&self) -> Vec<NodeOutputDef> {
        match self {
            NodeTypes::Output => vec![],
            _ => vec![NodeConnectionTypes::Texture2D.into()],
        }
    }
}

impl DataTypeTrait<GraphState> for NodeConnectionTypes {
    fn data_type_color(&self, _: &GraphState) -> egui::Color32 {
        let hue = match self {
            NodeConnectionTypes::Texture2D => 0.7,
            NodeConnectionTypes::None => 0.0,
        };

        Hsva::new(hue, 1., 1., 1.).into()
    }

    fn name(&self) -> std::borrow::Cow<str> {
        self.to_string().into()
    }
}

// const NODE_TYPES_VEC: Vec<NodeTypes> = ;

pub struct AllNodeTypes;
impl NodeTemplateIter for AllNodeTypes {
    type Item = NodeTypes;

    fn all_kinds(&self) -> Vec<Self::Item> {
        NodeTypes::get_all()
    }
}

impl UserResponseTrait for GraphResponse {}

impl NodeTemplateTrait for NodeTypes {
    type NodeData = NodeData;
    type DataType = NodeConnectionTypes;
    type ValueType = NodeValueTypes;

    fn node_finder_label(&self) -> &str {
        self.into()
    }

    fn node_graph_label(&self) -> String {
        self.node_finder_label().into()
    }

    fn user_data(&self) -> Self::NodeData {
        NodeData { template: self.clone(), result: None }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        node_id: NodeId
    ) {
        for input in self.get_input_types() {
            let connection = input.ty != NodeConnectionTypes::None;
            let value = input.value != NodeValueTypes::None;

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

//TODO: populating node graph & deps
//TODO: https://github.com/setzer22/egui_node_graph/blob/main/egui_node_graph_example/src/app.rs