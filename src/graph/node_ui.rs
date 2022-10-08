
use std::ops::{Sub, RangeInclusive};

use egui::{DragValue, color_picker::{color_edit_button_rgba}, Slider, color::Hsva};
use egui_node_graph::{Graph, NodeDataTrait, NodeId, WidgetValueTrait, DataTypeTrait};

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

fn horizontal_drags<const A: usize>(
    ui: &mut egui::Ui, 
    labels: &[&str; A],
    data: &mut NodeValueData<[f32; A]>
) -> egui::InnerResponse<bool> {

    ui.horizontal(|ui| {
        let mut any_changed = false;

        for i in 0..A {
            ui.label(labels[i].to_string());

            let speed = 0.1 * match data {
                NodeValueData{
                    min: Some(min),
                    max: Some(max),
                    ..
                } => {
                    (max[i]-min[i]).abs()
                }
                _ => {
                    1.0
                }
            };

            let range = default_range_f32(
                &data.min.map(|min| min[i]), 
                &data.max.map(|max| max[i])
            );

            let drag_value_ui = DragValue::new(&mut data.value[i])
                .speed(speed)
                .clamp_range(range);

            if ui.add(drag_value_ui).changed() {
                any_changed = true;
            }
        }

        any_changed
    })
}

fn default_range_f32(min: &Option<f32>, max: &Option<f32>) -> RangeInclusive<f32>{
    min.unwrap_or(0.0)..=max.unwrap_or(1.0)
}

fn default_range_i32(min: &Option<i32>, max: &Option<i32>) -> RangeInclusive<i32>{
    min.unwrap_or(0)..=max.unwrap_or(1)
}

impl WidgetValueTrait for NodeValueTypes {
    type Response = GraphResponse;

    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<Self::Response> {
        let _changed = match self {
            NodeValueTypes::Vec2 (data) => {
                ui.label(param_name);
                horizontal_drags(ui, &["x", "y"], data).inner
            }
            NodeValueTypes::Vec4(data) => {
                ui.label(param_name);
                horizontal_drags(ui, &["r", "g", "b", "a"], data).inner
            }
            NodeValueTypes::Color(NodeValueData { value, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    color_edit_button_rgba(ui, value, egui::color_picker::Alpha::OnlyBlend)
                }).inner.changed()
            }
            NodeValueTypes::Float (NodeValueData { value, min, max, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    // ui.add(DragValue::new(value))
                    ui.add(Slider::new(value, default_range_f32(min, max)).clamp_to_range(false))
                }).inner.changed()
            }
            NodeValueTypes::Long(NodeValueData { value, min, max, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value).clamp_range(default_range_i32(min, max)))
                }).inner.changed()
            },
            NodeValueTypes::Bool(NodeValueData { value, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.checkbox(value, "")
                }).inner.changed()
            }
            NodeValueTypes::Text(NodeValueData { value, .. }) => {
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