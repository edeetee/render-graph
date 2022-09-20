use std::{borrow::Cow, convert::{TryFrom, TryInto}};

use egui::{color::{Hsva}};
use egui_node_graph::{DataTypeTrait, NodeTemplateTrait, Graph, NodeId, InputId, OutputId, NodeTemplateIter, UserResponseTrait};
use isf::{Input, InputType};
use itertools::Itertools;
use strum::IntoEnumIterator;

use super::{def::*, connection_types::{NodeInputDef, NodeOutputDef, DEFAULT_TEXTURE2D_INPUT}, isf::parse_isf_shaders};

impl <'a> From<&'a NodeTypes> for &'a str {
    fn from(ty: &'a NodeTypes) -> Self {
        match ty {
            NodeTypes::Instances => "Instances",
            NodeTypes::Output => "Output",
            // NodeTypes::Feedback => "Feedback",
            // NodeTypes::Sdf => "Sdf",
            // NodeTypes::Uv => "UV",
            NodeTypes::Isf{name, ..} => name
        }
    }
}

impl NodeTypes {
    pub fn get_types() -> Vec<NodeTypes> {
        let shaders = parse_isf_shaders()
            .map(|(name, isf)| NodeTypes::Isf{name, isf});

        let mut types = vec![
            NodeTypes::Instances,
            // NodeTypes::Feedback,
            // NodeTypes::Sdf,
            // NodeTypes::Uv,
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
                isf.inputs.iter().filter_map(|input| input.try_into().ok()).collect()
            }
            _ => vec![DEFAULT_TEXTURE2D_INPUT],
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
            // NodeConnectionTypes::FrameBuffer => 0.0,
            NodeConnectionTypes::Texture2D => 0.7,
            NodeConnectionTypes::Float => 0.2,
            NodeConnectionTypes::Vec2 => 0.4,
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
        NodeTypes::get_types()
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
            let isf_ty = input.value.0;

            let kind = match isf_ty {
                InputType::Image => egui_node_graph::InputParamKind::ConnectionOnly,
                InputType::Float(_) => egui_node_graph::InputParamKind::ConnectionOrConstant,
                _ => egui_node_graph::InputParamKind::ConstantOnly,
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