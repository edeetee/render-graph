
use std::{ops::{RangeInclusive}, path::Path};

use egui::{DragValue, color_picker::{color_edit_button_rgba}, Slider, color::Hsva, RichText, Color32, Stroke, Label, Sense};
use egui_node_graph::{Graph, NodeDataTrait, NodeId, WidgetValueTrait, DataTypeTrait};


use super::def::*;

fn draw_error(ui: &mut egui::Ui, name: &str, error: &Option<NodeError>){
    if let Some(error) = &error {
        egui::Frame::none()
            .inner_margin(2.0)
            .stroke(Stroke::new(1.0, Color32::RED))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.label(RichText::new(format!("ERROR in {name}")).code().color(Color32::RED));
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
        _state: &Self::UserState,
    ) -> Vec<egui_node_graph::NodeResponse<Self::Response, Self>>
    where
        Self::Response: egui_node_graph::UserResponseTrait,
    {
        let node = &graph[node_id];

        egui::Frame::none()
            .stroke(Stroke::new(1.0, Color32::LIGHT_GRAY))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());

                if let Some(tex) = &node.user_data.texture.upgrade() {
                    let width = ui.available_width();
                    let tex = tex.borrow();
                    let (tex_w, tex_h) = tex.size();
                    let height = tex_h as f32 * width / tex_w as f32;
        
                    ui.image(tex.clone_screen_tex_id(), [width, height]);
                } else {
                    ui.label("NO IMAGE AVAILABLE");
                }

            });
        

        draw_error(ui, "Init", &node.user_data.create_error);
        draw_error(ui, "Update", &node.user_data.update_error);
        draw_error(ui, "Render", &node.user_data.render_error);
        
        vec![]
    }
}

impl DataTypeTrait<GraphState> for ConnectionType {
    fn data_type_color(&self, _: &GraphState) -> egui::Color32 {
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

fn horizontal_drags<const A: usize>(
    ui: &mut egui::Ui, 
    labels: &[&str; A],
    data: &mut RangedData<[f32; A]>
) -> egui::InnerResponse<bool> {

    ui.horizontal(|ui| {
        let mut any_changed = false;

        for i in 0..A {
            ui.label(labels[i].to_string());

            let speed = 0.01 * match data {
                RangedData{
                    min: Some(min),
                    max: Some(max),
                    ..
                } => {
                    (max[i]-min[i]).abs()
                }
                _ => {
                    10.0
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

impl WidgetValueTrait for UiValue {
    type Response = GraphResponse;

    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<Self::Response> {

        let _changed = match self {

            UiValue::Vec2 (data) => {
                ui.label(param_name);
                horizontal_drags(ui, &["x", "y"], data).inner
            }

            UiValue::Vec4(data) => {
                ui.label(param_name);
                horizontal_drags(ui, &["r", "g", "b", "a"], data).inner
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

                    ui.horizontal(|ui| {
                        ui.label("tx");
                        changed |= ui.add(DragValue::new(&mut mat.translation.x)).changed();

                        ui.label("ty");
                        changed |= ui.add(DragValue::new(&mut mat.translation.y)).changed();

                        ui.label("tz");
                        changed |= ui.add(DragValue::new(&mut mat.translation.z)).changed();
                    });

                    ui.horizontal(|ui| {
                        ui.label("rx");
                        changed |= ui.drag_angle(&mut mat.rotation.0).changed();

                        ui.label("ry");
                        changed |= ui.drag_angle(&mut mat.rotation.1).changed();

                        ui.label("rz");
                        changed |= ui.drag_angle(&mut mat.rotation.2).changed();
                    });
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