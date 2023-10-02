use std::{ops::RangeInclusive, path::Path};

use egui::{
    color_picker::color_edit_button_rgba, Align, Area, Color32, DragValue, Frame, Id,
    InnerResponse, Layout, Order, Response, Rgba, Slider, Stroke, Ui, Widget,
};
use egui_node_graph::{NodeId, WidgetValueTrait};
use serde::{Deserialize, Serialize};

use crate::common::{
    animation::DataUpdater,
    def::{RangedData, Reset, TextStyle, UiValue},
};
use crate::graph::def::{GraphResponse, GraphState, UiNodeData};
use crate::widgets::limited_ui::{horizontal_drags, UiLimit};

fn default_range_f32(min: &Option<f32>, max: &Option<f32>) -> RangeInclusive<f32> {
    min.unwrap_or(0.0)..=max.unwrap_or(1.0)
}

fn default_range_i32(min: &Option<i32>, max: &Option<i32>) -> RangeInclusive<i32> {
    min.unwrap_or(0)..=max.unwrap_or(1)
}

#[derive(Serialize, Deserialize)]
pub enum UpdaterUiState {
    None,
    Editing,
}

struct ParamUiResponse {
    response: Response,
    changed: bool,
}

impl From<InnerResponse<Response>> for ParamUiResponse {
    fn from(value: InnerResponse<Response>) -> Self {
        Self {
            changed: value.response.changed() || value.inner.changed(),
            response: value.response,
        }
    }
}

impl From<InnerResponse<bool>> for ParamUiResponse {
    fn from(value: InnerResponse<bool>) -> Self {
        Self {
            changed: value.response.changed() || value.inner,
            response: value.response,
        }
    }
}

impl From<Response> for ParamUiResponse {
    fn from(value: Response) -> Self {
        Self {
            changed: value.changed(),
            response: value,
        }
    }
}

pub fn popup<R>(
    ui: &Ui,
    popup_id: Id,
    widget_response: &Response,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    Area::new(popup_id)
        .order(Order::Foreground)
        .fixed_pos(widget_response.rect.left_bottom())
        .show(ui.ctx(), |ui| {
            // Note: we use a separate clip-rect for this area, so the popup can be outside the parent.
            // See https://github.com/emilk/egui/issues/825
            let frame = Frame::popup(ui.style());
            let _frame_margin = frame.inner_margin + frame.outer_margin;
            frame
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                        // ui.set_wisdth(widget_response.rect.width() - frame_margin.sum().x);
                        add_contents(ui)
                    })
                    .inner
                })
                .inner
        })
}

fn draw_param(param: &mut UiValue, ui: &mut Ui, param_name: &str) -> ParamUiResponse {
    match param {
        UiValue::Vec2(data) => {
            ui.label(param_name);
            horizontal_drags(
                ui,
                &["x", "y"],
                UiLimit::Clamp(data.min.as_ref(), data.max.as_ref()),
                &mut data.value,
            )
            .into()
        }

        UiValue::Vec4(data) => {
            ui.label(param_name);
            horizontal_drags(
                ui,
                &["r", "g", "b", "a"],
                UiLimit::Clamp(data.min.as_ref(), data.max.as_ref()),
                &mut data.value,
            )
            .into()
        }

        UiValue::Color(RangedData { value, .. }) => ui
            .horizontal(|ui| {
                ui.label(param_name);
                color_edit_button_rgba(
                    ui,
                    unsafe { std::mem::transmute::<&mut [f32; 4], &mut Rgba>(value) },
                    egui::color_picker::Alpha::OnlyBlend,
                )
            })
            .into(),

        UiValue::Float(RangedData {
            value, min, max, ..
        }) => {
            ui.horizontal(|ui| {
                ui.label(param_name);
                // ui.add(DragValue::new(value))
                ui.add(
                    Slider::new(value, default_range_f32(min, max))
                        .clamp_to_range(false)
                        .fixed_decimals(2),
                )
            })
            .into()
        }

        UiValue::Long(RangedData {
            value, min, max, ..
        }) => ui
            .horizontal(|ui| {
                ui.label(param_name);
                ui.add(DragValue::new(value).clamp_range(default_range_i32(min, max)))
            })
            .into(),

        UiValue::Menu(RangedData { value, .. }, ref label_mapping) => ui
            .horizontal_wrapped(|ui| {
                ui.label(param_name);

                let mut changed = false;

                for (label, val_for_label) in label_mapping {
                    if ui.selectable_label(val_for_label == value, label).clicked() {
                        *value = *val_for_label;
                        changed = true;
                    }
                }

                changed
            })
            .into(),

        UiValue::Bool(RangedData { value, .. }) => ui
            .horizontal(|ui| {
                ui.label(param_name);
                ui.checkbox(value, "")
            })
            .into(),

        UiValue::Path(path) => {
            ui.horizontal(|ui| {
                ui.label(param_name);

                let path_text = if let Some(path) = path {
                    if let Some(path_str) = path.to_str() {
                        let max_length = 30;

                        if max_length < path_str.len() {
                            &path_str[path_str.len() - max_length..]
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
            })
            .into()
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

                changed |=
                    horizontal_drags(ui, &["tx", "ty", "tz"], UiLimit::None, &mut mat.translation)
                        .inner;

                // mat.translation = Vec3::from_slice(&slice);
                changed |= horizontal_drags(
                    ui,
                    &["rx", "ry", "rz"],
                    UiLimit::Loop(&[0f32; 3], &[360f32; 3]),
                    &mut mat.rotation,
                )
                .inner;

                if changed {
                    mat.update_mat();
                }

                changed
            })
            .into()
        }

        UiValue::Text(RangedData { value, .. }, style) => ui
            .horizontal(|ui| {
                ui.label(param_name);
                let widget = match style {
                    TextStyle::Oneline => egui::TextEdit::singleline(value),
                    TextStyle::Multiline => egui::TextEdit::multiline(value).code_editor(),
                };
                ui.set_max_width(256.0);
                ui.add_sized(ui.available_size(), widget)
            })
            .into(),

        UiValue::None => ui.label(param_name).into(),
    }
}

impl WidgetValueTrait for UiValue {
    type Response = GraphResponse;
    type UserState = GraphState;
    type NodeData = UiNodeData;

    fn value_widget(
        &mut self,
        param_name: &str,
        node_id: NodeId,
        ui: &mut egui::Ui,
        user_state: &mut Self::UserState,
        _node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {
        let param_key = (node_id, param_name.to_string());

        let is_animating = user_state.animations.contains_key(&param_key);

        let param_frame_color = if is_animating {
            Color32::LIGHT_GRAY
        } else {
            Color32::TRANSPARENT
        };

        let param_response: ParamUiResponse = Frame::none()
            // .rounding(2.0)
            .inner_margin(4.0)
            .stroke(Stroke {
                width: 1.0,
                color: param_frame_color,
            })
            .show(ui, |ui| draw_param(self, ui, param_name))
            .inner;

        let animator_popup_id = ui.make_persistent_id(param_key.clone());

        if ui.rect_contains_pointer(param_response.response.rect)
            && ui.input().pointer.secondary_clicked()
        {
            user_state.param_with_popup = Some(param_key.clone());
        }

        if user_state.param_with_popup.as_ref() == Some(&param_key) {
            let popup_response = popup(&ui, animator_popup_id, &param_response.response, |ui| {
                ui.vertical(|ui| {
                    if ui.button("RESET").clicked() {
                        self.reset();
                    }

                    ui.horizontal(|ui| {
                        let animator = user_state.animations.get_mut(&param_key);
                        let mut delete = false;

                        match animator {
                            Some(updater) => {
                                delete |= ui.button("REMOVE").clicked();
                                updater.ui(ui);
                            }
                            None => {
                                if ui.button("ANIMATE").clicked() {
                                    if let Some(updater) = DataUpdater::from_param(self) {
                                        user_state.animations.insert(param_key.clone(), updater);
                                    }
                                }
                            }
                        }

                        if delete {
                            user_state.animations.remove(&param_key);
                        }
                    })
                })
            });

            if (popup_response.response.clicked_elsewhere()
                && param_response.response.clicked_elsewhere())
                || param_response.response.clicked()
            {
                user_state.param_with_popup = None;
            }
        }

        if param_response.changed {
            user_state.param_with_popup = None;
        }

        // if let Some(popup_response) = popup_response {
        //     if ui.rect_contains_pointer(popup_response.response.rect) {
        //         ui.memory().open_popup(animator_popup_id);
        //     }
        // }

        // if let Some(popup_response) = popup_response {
        //     let input = ui.input();
        //     if input.pointer.primary_clicked() && !ui.rect_contains_pointer(popup_response.response.rect) {
        //         user_state.editing_param = None;
        //     }
        // }

        vec![]
    }
}
