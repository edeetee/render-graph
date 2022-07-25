use std::{borrow::Cow};

use egui::color::{Hsva};
use egui_node_graph::{DataTypeTrait, NodeTemplateTrait, Graph, NodeId, InputId, OutputId, NodeTemplateIter};

use super::{def::*, helpers::GraphHelper};

impl DataTypeTrait<GraphState> for NodeConnectionTypes {
    fn data_type_color(&self, _: &GraphState) -> egui::Color32 {
        let hue = match self {
            NodeConnectionTypes::FrameBuffer => 0.0,
            NodeConnectionTypes::Texture2D => 0.7,
        };

        Hsva::new(hue, 1., 0., 1.).into()
    }

    fn name(&self) -> std::borrow::Cow<str> {
        Cow::Borrowed(self.into())
    }
}



impl GraphHelper<NodeConnectionTypes> for Graph<NodeData, NodeConnectionTypes, ValueTypes> {
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

impl NodeTemplateTrait for NodeTypes {
    type NodeData = NodeData;
    type DataType = NodeConnectionTypes;
    type ValueType = ValueTypes;

    fn node_finder_label(&self) -> &str {
        self.into()
    }

    fn node_graph_label(&self) -> String {
        self.node_finder_label().into()
    }

    fn user_data(&self) -> Self::NodeData {
        NodeData { template: *self }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        node_id: NodeId
    ) {
        
        match self {
            NodeTypes::Instances => {
                graph.input(node_id, NodeConnectionTypes::FrameBuffer);
                graph.output(node_id, NodeConnectionTypes::Texture2D);
            },
            NodeTypes::Feedback => {
                graph.in_out(node_id, NodeConnectionTypes::Texture2D);
            },
            NodeTypes::Sdf => {
                graph.in_out(node_id, NodeConnectionTypes::Texture2D);
            },
            NodeTypes::Output => {
                graph.input(node_id, NodeConnectionTypes::Texture2D);
            },
        }
    }
}