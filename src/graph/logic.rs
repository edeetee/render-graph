

use egui::{color::{Hsva}};
use egui_node_graph::{DataTypeTrait, NodeTemplateTrait, Graph, NodeId, NodeTemplateIter, UserResponseTrait};

use super::{def::*, connection_types::{NodeInputDef, NodeOutputDef}, isf::parse_isf_shaders};

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

impl UserResponseTrait for GraphResponse {}


//TODO: populating node graph & deps
//TODO: https://github.com/setzer22/egui_node_graph/blob/main/egui_node_graph_example/src/app.rs