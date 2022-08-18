use std::{borrow::Cow};

use egui::{color::{Hsva}};
use egui_node_graph::{DataTypeTrait, NodeTemplateTrait, Graph, NodeId, InputId, OutputId, NodeTemplateIter, UserResponseTrait};
use strum::IntoEnumIterator;

use super::{def::*, util::GraphMutHelper};

impl From<&NodeConnectionTypes> for NodeValueTypes {
    fn from(connection: &NodeConnectionTypes) -> Self {
        match connection {
            // NodeConnectionTypes::FrameBuffer => NodeValueTypes::None,
            NodeConnectionTypes::Texture2D => NodeValueTypes::None,
        }
    }
}

impl DataTypeTrait<GraphState> for NodeConnectionTypes {
    fn data_type_color(&self, _: &GraphState) -> egui::Color32 {
        let hue = match self {
            // NodeConnectionTypes::FrameBuffer => 0.0,
            NodeConnectionTypes::Texture2D => 0.7,
        };

        Hsva::new(hue, 1., 1., 1.).into()
    }

    fn name(&self) -> std::borrow::Cow<str> {
        Cow::Borrowed(self.into())
    }
}

impl GraphMutHelper<NodeConnectionTypes> for Graph<NodeData, NodeConnectionTypes, NodeValueTypes> {
    fn input_named(&mut self, node_id: NodeId, connection: NodeConnectionTypes, name: &str) -> InputId {
        let value = (&connection).into();

        self.add_input_param(
            node_id, 
            name.into(), 
            connection, 
            value, 
            egui_node_graph::InputParamKind::ConnectionOnly, 
            true
        )
    }

    fn output_named(&mut self, node_id: NodeId, connection: NodeConnectionTypes, name: &str) -> OutputId {
        self.add_output_param(node_id, name.into(), connection)
    }
}

// const NODE_TYPES_VEC: Vec<NodeTypes> = ;

pub struct AllNodeTypes;
impl NodeTemplateIter for AllNodeTypes {
    type Item = NodeTypes;

    fn all_kinds(&self) -> Vec<Self::Item> {
        NodeTypes::iter().collect()
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
        NodeData::new(*self)
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        node_id: NodeId
    ) {
        match self {
            NodeTypes::Output => {
                graph.input(node_id, NodeConnectionTypes::Texture2D);
            },
            _ => {
                graph.in_out(node_id, self.into());
            },
        }
    }
}

//TODO: populating node graph & deps
//TODO: https://github.com/setzer22/egui_node_graph/blob/main/egui_node_graph_example/src/app.rs