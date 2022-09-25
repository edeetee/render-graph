
use egui::{DragValue, color_picker::{color_edit_button_rgba}, Slider};
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
            let size = ui.available_width();
            ui.image(*tex_id, [size, size]);
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
            NodeValueTypes::Vec2 (value) => {
                ui.label(param_name);

                ui.horizontal(|ui| {
                    ui.label("x");
                    let x_response = ui.add(DragValue::new(&mut value[0]).speed(0.1));
                    ui.label("y");
                    let y_response = ui.add(DragValue::new(&mut value[1]).speed(0.1));

                    x_response.changed() || y_response.changed()
                }).inner
            }
            NodeValueTypes::Vec4(value) => {
                ui.label(param_name);

                ui.horizontal(|ui| {
                    ui.label("r");
                    let r = &ui.add(DragValue::new(&mut value[0]).speed(0.1));
                    ui.label("g");
                    let g = &ui.add(DragValue::new(&mut value[1]).speed(0.1));
                    ui.label("b");
                    let b = &ui.add(DragValue::new(&mut value[2]).speed(0.1));
                    ui.label("a");
                    let a = &ui.add(DragValue::new(&mut value[3]).speed(0.1));

                    vec![r,g,b,a].iter().any(|resp| resp.changed())
                }).inner
            }
            NodeValueTypes::Color(value) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    // let rgba = Rgba::from_rgba_premultiplied(value);
                    color_edit_button_rgba(ui, value, egui::color_picker::Alpha::OnlyBlend)
                    // ui.add()
                }).inner.changed()
            }
            NodeValueTypes::Float (value) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    // ui.add(DragValue::new(value))
                    ui.add(Slider::new(value, 0f32..=1f32).clamp_to_range(false))
                }).inner.changed()
            }
            NodeValueTypes::Bool(value) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.checkbox(value, "")
                }).inner.changed()
            }
            NodeValueTypes::Text(value) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.text_edit_singleline(value)
                }).inner.changed()
            }
            NodeValueTypes::None => { false }
        };

        vec![]
    }
}