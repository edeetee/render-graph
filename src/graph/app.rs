use eframe::CreationContext;
use egui::DragValue;
use egui_node_graph::{Graph, NodeDataTrait, NodeId, WidgetValueTrait, GraphEditorState};

use super::{def::*, logic::AllNodeTypes};

impl NodeDataTrait for NodeData {
    type Response = GraphResponse;
    type UserState = GraphState;
    type DataType = NodeConnectionTypes;
    type ValueType = NodeValueTypes;

    fn bottom_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &Self::UserState,
    ) -> Vec<egui_node_graph::NodeResponse<Self::Response, Self>>
    where
        Self::Response: egui_node_graph::UserResponseTrait,
    {
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

// -------------------------------------------------------------------

type EditorState = GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;

pub struct NodeGraphApp{
    pub state: EditorState,
}

impl NodeGraphApp {
    pub fn new(cc: &CreationContext) -> Self {
        Self { 
            state: GraphEditorState::new(1.0, GraphState {})
        }
    }
}

impl eframe::App for NodeGraphApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("Hello World!");
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.state.draw_graph_editor(ui, AllNodeTypes);

            // ui.image(texture_id, size)
        });
    }
}
