use std::{cell::RefCell, rc::Weak};

use egui::{color::Hsva, Color32, Label, Response, RichText, Sense, Stroke, Ui};
use egui_node_graph::{DataTypeTrait, Graph, NodeDataTrait, NodeId};

use super::{def::*, ui_texture::UiTexture};

fn draw_error(ui: &mut egui::Ui, name: &str, error: &Option<graph::NodeError>) {
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
                ui.label(
                    RichText::new(format!("Error in {name}"))
                        .code()
                        .color(Color32::LIGHT_RED),
                );
                ui.label(RichText::new(format!("{err_elapsed_s:.2}s ago")).small());
                ui.add(
                    Label::new(RichText::new(&error.text).code()).sense(Sense::click_and_drag()),
                );
            });
    }
}

enum ImageScale {
    MaxWidth(f32),
    MaxSize(f32),
}

fn show_image(ui: &mut Ui, texture: Weak<RefCell<UiTexture>>, scale: ImageScale) -> Response {
    egui::Frame::none()
        .stroke(Stroke::new(1.0, Color32::BLACK))
        .show(ui, |ui| {
            // ui.set_min_size(ui.available_size());

            if let Some(tex) = texture.upgrade() {
                let tex = tex.borrow();

                let (tex_w, tex_h) = tex.size();
                let tex_size = glam::Vec2::new(tex_w as f32, tex_h as f32);

                let img_size = match scale {
                    ImageScale::MaxWidth(width) => {
                        let height = tex_size.y * width / tex_size.x;
                        glam::Vec2::new(width, height)
                    }
                    ImageScale::MaxSize(max_size) => {
                        glam::Vec2::new(tex_size.x, tex_size.y).clamp_length_max(max_size)
                    }
                };

                ui.image(tex.id(), img_size.to_array())
            } else {
                ui.label("NO IMAGE AVAILABLE")
            }
        })
        .response
}

impl NodeDataTrait for UiNodeData {
    type Response = CustomGraphResponse;
    type UserState = GraphState;
    type DataType = ConnectionType;
    type ValueType = UiValue;

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        state: &mut Self::UserState,
    ) -> Vec<egui_node_graph::NodeResponse<Self::Response, Self>>
    where
        Self::Response: egui_node_graph::UserResponseTrait,
    {
        ui.set_width(256.0);
        let node = &graph[node_id];

        let tex_expanded = state.visible_nodes.contains(&node_id);

        if tex_expanded {
            if show_image(
                ui,
                node.user_data.texture.clone(),
                ImageScale::MaxWidth(ui.available_width()),
            )
            .interact(egui::Sense::click())
            .clicked()
            {
                state.visible_nodes.remove(&node_id);
            };
        } else {
            if show_image(
                ui,
                node.user_data.texture.clone(),
                ImageScale::MaxSize(50.0),
            )
            .interact(egui::Sense::click())
            .clicked()
            {
                state.visible_nodes.insert(node_id);
            }
        }

        if ui.ui_contains_pointer() {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("img_hover"), |ui| {
                show_image(
                    ui,
                    node.user_data.texture.clone(),
                    ImageScale::MaxSize(200.0),
                )
            });
        }

        draw_error(ui, "Init", &node.user_data.create_error);
        draw_error(ui, "Update", &node.user_data.update_error);
        draw_error(ui, "Render", &node.user_data.render_error);

        draw_time(ui, node.user_data.render_time);

        vec![]
    }
}

fn draw_time(ui: &mut egui::Ui, time: Option<std::time::Duration>) {
    if let Some(time) = time {
        let time_us = time.as_micros();

        let color = if time_us < 100 {
            Color32::GREEN
        } else if time_us < 1000 {
            Color32::YELLOW
        } else {
            Color32::RED
        };

        ui.colored_label(color, format!("{time_us}μs"));
    }
}
