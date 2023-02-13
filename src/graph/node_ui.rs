
use std::{ops::{RangeInclusive, RangeBounds, Bound}, path::Path};

use egui::{DragValue, color_picker::{color_edit_button_rgba}, Slider, color::Hsva, RichText, Color32, Stroke, Label, Sense};
use egui_node_graph::{Graph, NodeDataTrait, NodeId, WidgetValueTrait, DataTypeTrait};
use glam::Vec3;


use crate::common::def::{ConnectionType, UiValue, RangedData, TextStyle, Mat4UiData};

use super::def::*;

fn draw_error(ui: &mut egui::Ui, name: &str, error: &Option<NodeError>){
    if let Some(error) = &error {

        // let err_time_diff = error.when.elapsed();
        let err_elapsed_s = error.when.elapsed().as_secs_f32();
        // error.when.elapsed()

        let error_is_recent = err_elapsed_s < 1.0;

        let color = if error_is_recent {
            Color32::RED
        } else {
            Color32::GRAY
        };

        egui::Frame::none()
            .inner_margin(2.0)
            .stroke(Stroke::new(1.0, color))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.label(RichText::new(format!("Error in {name}")).code().color(Color32::LIGHT_RED));
                ui.label(RichText::new(format!("{err_elapsed_s:.2}s ago")).small());
                ui.add(Label::new(RichText::new(&error.text).code()).sense(Sense::click_and_drag()));
            });
    }
}

impl NodeDataTrait for NodeData {
    type Response = GraphResponse;
    type UserState = GraphState;
    type DataType = ConnectionType;
    type ValueType = UiValue;

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _state: &mut Self::UserState,
    ) -> Vec<egui_node_graph::NodeResponse<Self::Response, Self>>
    where
        Self::Response: egui_node_graph::UserResponseTrait,
    {
        let node = &graph[node_id];

        // if ui.ui_contains_pointer() {
        //     egui::Area
        // }
        if ui.ui_contains_pointer() {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("img_hover"), |ui| {
                egui::Frame::none()
                    .stroke(Stroke::new(1.0, Color32::LIGHT_GRAY))
                    .show(ui, |ui| {
                        // ui.set_min_size(ui.available_size());
    
                        if let Some(tex) = &node.user_data.texture.upgrade() {
                            let width = 200.0;
                            let tex = tex.borrow();
                            let (tex_w, tex_h) = tex.size();
                            let height = tex_h as f32 * width / tex_w as f32;
                
                            ui.image(tex.clone_screen_tex_id(), [width, height]);
                        } else {
                            ui.label("NO IMAGE AVAILABLE");
                        }
                    });
            });
        }

        draw_error(ui, "Init", &node.user_data.create_error);
        draw_error(ui, "Update", &node.user_data.update_error);
        draw_error(ui, "Render", &node.user_data.render_error);
        
        vec![]
    }
}

impl DataTypeTrait<GraphState> for ConnectionType {
    fn data_type_color(&self, _: &mut GraphState) -> egui::Color32 {
        let hue = match self {
            ConnectionType::Texture2D => 0.7,
            ConnectionType::None => 0.0,
        };

        Hsva::new(hue, 1., 1., 1.).into()
    }

    fn name(&self) -> std::borrow::Cow<str> {
        self.to_string().into()
    }
}

fn get_val<T>(bound: Bound<T>) -> Option<T> {
    match bound {
        Bound::Included(v) => Some(v),
        Bound::Excluded(v) => Some(v),
        Bound::Unbounded => None,
    }
}

enum UiLimit<T> {
    Clamp(Option<T>, Option<T>),
    Loop(T, T),
    None
}

impl <T> UiLimit<T> {
    fn min(&self) -> Option<&T> {
        match self {
            UiLimit::Clamp(min, _) => min.as_ref(),
            UiLimit::Loop(min, _) => Some(min),
            UiLimit::None => None,
        }
    }

    fn max(&self) -> Option<&T> {
        match self {
            UiLimit::Clamp(_, max) => max.as_ref(),
            UiLimit::Loop(_, max) => Some(max),
            UiLimit::None => None,
        }
    }
}

fn horizontal_drags<const A: usize>(
    ui: &mut egui::Ui, 
    labels: &[&str; A],
    limits: UiLimit<&[f32; A]>,
    values: &mut [f32; A],
) -> egui::InnerResponse<bool> {

    ui.horizontal(|ui| {
        let mut any_changed = false;

        for i in 0..A {
            // let range = &ranges[i];
            // let value = &mut values[i];
            let label = labels[i];

            ui.label(label);

            let min = limits.min().map(|min| min[i]);
            let max = limits.max().map(|max| max[i]);

            let speed =  match (min, max) {
                (Some(min), Some(max)) => 0.01 * (max - min).abs(),
                _ => 0.1
            };

            let drag_value_ui = DragValue::new(&mut values[i])
                .speed(speed);

            if ui.add(drag_value_ui).changed() {
                any_changed = true;
            }

            match limits {
                UiLimit::Loop(min, max) => {
                    let sum = (max[i] - min[i]).abs();
    
                    let mut temp_val = values[i];
    
                    //center at 0
                    temp_val -= min[i];
                    temp_val %= sum;
                    temp_val += min[i];
    
                    values[i] = temp_val;
                },

                UiLimit::Clamp(_, _) => {
                    if let Some(min) = min {
                        values[i] = values[i].max(min);
                    }
        
                    if let Some(max) = max {
                        values[i] = values[i].min(max);
                    }
                },
                UiLimit::None => {},
            }
        }

        any_changed
    })
}

// fn horizontal_drags_arr()

fn default_range_f32(min: &Option<f32>, max: &Option<f32>) -> RangeInclusive<f32>{
    min.unwrap_or(0.0)..=max.unwrap_or(1.0)
}

fn default_range_i32(min: &Option<i32>, max: &Option<i32>) -> RangeInclusive<i32>{
    min.unwrap_or(0)..=max.unwrap_or(1)
}

impl WidgetValueTrait for UiValue {
    type Response = GraphResponse;
    type UserState = GraphState;
    type NodeData = NodeData;

    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut Self::UserState,
        _node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {

        let _changed = match self {

            UiValue::Vec2 (data) => {
                ui.label(param_name);
                horizontal_drags(
                    ui, 
                    &["x", "y"], 
                    UiLimit::Clamp(data.min.as_ref(), data.max.as_ref()),
                    // ,
                    // ,
                    &mut data.value, 
                ).inner
            }

            UiValue::Vec4(data) => {
                ui.label(param_name);
                horizontal_drags(
                    ui, 
                    &["r", "g", "b", "a"], 
                    UiLimit::Clamp(data.min.as_ref(), data.max.as_ref()),
                    &mut data.value, 
                ).inner
            }

            UiValue::Color(RangedData { value, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    color_edit_button_rgba(ui, value, egui::color_picker::Alpha::OnlyBlend)
                }).inner.changed()
            }

            UiValue::Float (RangedData { value, min, max, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    // ui.add(DragValue::new(value))
                    ui.add(Slider::new(value, default_range_f32(min, max)).clamp_to_range(false))
                }).inner.changed()
            }

            UiValue::Long(RangedData { value, min, max, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value).clamp_range(default_range_i32(min, max)))
                }).inner.changed()
            },

            UiValue::Bool(RangedData { value, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.checkbox(value, "")
                }).inner.changed()
            }

            UiValue::Path(path) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);

                    let path_text = if let Some(path) = path {
                        if let Some(path_str) = path.to_str() {
                            let max_length = 30;

                            if max_length < path_str.len() {
                                &path_str[path_str.len()-max_length..]
                            } else {
                                path_str
                            }
                        } else {
                            "???"
                        }
                    } else {
                        "Open"
                    };
                    let open_resp = ui.button(path_text);

                    if ui.ui_contains_pointer() {
                        let files = &ui.ctx().input().raw.dropped_files;
                        if let Some(file) = files.iter().next() {
                            if file.path.is_some() {
                                *path = file.path.clone();
                            }
                        }
                    }

                    if open_resp.clicked() {
                        let open_dir = path.as_deref().map(Path::to_str).flatten().unwrap_or(&"~");

                        let new_path = native_dialog::FileDialog::new()
                            .set_location(open_dir)
                            .add_filter("OBJ file", &["obj"])
                            // .add_filter("JPEG Image", &["jpg", "jpeg"])
                            .show_open_single_file()
                            .unwrap();

                        if new_path.is_some() {
                            *path = new_path;
                        }
                    }

                    open_resp
                }).inner.changed()
            }

            UiValue::Mat4(mat) => {
                let mut changed = false;

                ui.vertical(|ui| {
                    ui.label(param_name);

                    ui.horizontal(|ui| {
                        ui.label("s");
                        changed |= ui.add(DragValue::new(&mut mat.scale)).changed()
                    });

                    // let tx
                    // let mut slice = mat.translation.to_array();

                    changed |= horizontal_drags(
                        ui, 
                        &["tx", "ty", "tz"], 
                        UiLimit::None,
                        &mut mat.translation
                    ).inner;

                    // mat.translation = Vec3::from_slice(&slice);
                    changed |= horizontal_drags(
                        ui, 
                        &["rx", "ry", "rz"], 
                        UiLimit::Loop(&[0f32; 3], &[360f32; 3]),
                        &mut mat.rotation
                    ).inner;
                });

                if changed {
                    mat.update_mat();
                }

                changed
            }

            UiValue::Text(RangedData { value, .. }, style) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    let widget = match style {
                        TextStyle::Oneline => egui::TextEdit::singleline(value),
                        TextStyle::Multiline => egui::TextEdit::multiline(value).code_editor()
                    };
                    ui.set_max_width(256.0);
                    ui.add_sized(ui.available_size(), widget)
                }).inner.changed()
            }

            UiValue::None => { ui.label(param_name); false }
        };

        vec![]
    }
}

fn draw_matrix(_ui: &egui::Ui, _v: &mut Mat4UiData) {

}