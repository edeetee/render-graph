
use egui::{DragValue};
use egui_node_graph::{Graph, NodeDataTrait, NodeId, WidgetValueTrait};

use super::def::*;

impl NodeDataTrait for NodeData {
    type Response = GraphResponse;
    type UserState = GraphState;
    type DataType = NodeConnectionTypes;
    type ValueType = NodeValueTypes;

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _state: &Self::UserState,
    ) -> Vec<egui_node_graph::NodeResponse<Self::Response, Self>>
    where
        Self::Response: egui_node_graph::UserResponseTrait,
    {
        let me = &graph[node_id];

        if let Some(tex_id) = &me.user_data.result {
            ui.image(*tex_id, ui.available_size());
        } else {
            ui.label("NO IMAGE AVAILABLE");
        }
        
        vec![]
    }
}

impl WidgetValueTrait for NodeValueTypes {
    type Response = GraphResponse;

    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<Self::Response> {
        match self {
            NodeValueTypes::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut value[0]));
                    ui.label("y");
                    ui.add(DragValue::new(&mut value[1]));
                });
            }
            NodeValueTypes::Float { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
            NodeValueTypes::None => {}
        }

        Vec::new()
    }
}