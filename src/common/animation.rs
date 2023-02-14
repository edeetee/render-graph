use std::time::Duration;

use egui::{Ui, Widget};
use glam::{Vec3, Vec3A, Mat4, Quat};
use serde::{Serialize, Deserialize};
use tri_mesh::prelude::Quaternion;

use super::{def::UiValue, ui_util::horizontal_drags};


#[derive(Debug, Serialize, Deserialize)]
pub struct RotationAnimation {
    axis: [f32; 3],
    speed: f32
}

impl Widget for &mut RotationAnimation {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.label("axis");
            let axis_resp = horizontal_drags(ui, &["x", "y", "z"], super::ui_util::UiLimit::None, &mut self.axis);

            if axis_resp.inner {
                Vec3::from_slice(&self.axis)
                    .try_normalize()
                    .unwrap_or(Vec3::Y)
                    .write_to_slice(&mut self.axis);
            }

            ui.label("speed");
            ui.add(egui::Slider::new(&mut self.speed, -1.0..=1.0));
        }).response
    }
}

pub struct UpdateInfo {
    elapsed_since_update: Duration,
    seconds_since_update: f32
}

impl UpdateInfo {
    pub fn new(elapsed_since_update: Duration) -> Self {
        Self {
            seconds_since_update: elapsed_since_update.as_secs_f32(),
            elapsed_since_update
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataUpdater {
    ///Changes this per second
    FloatSpeed(f32),
    Rotation(RotationAnimation)
}

impl DataUpdater {
    pub fn from_param(val: &UiValue) -> Option<Self> {
        match val {
            UiValue::Mat4(_) => Some(DataUpdater::Rotation(RotationAnimation { axis: Vec3::Y.to_array(), speed: 0.0 })),
            UiValue::Float(_) => Some(DataUpdater::FloatSpeed(0.0)),
            _ => None
        }
    }

    pub fn update_value(&self, val: &mut UiValue, info: &UpdateInfo) {
        match (self, val) {
            (DataUpdater::Rotation(anim), UiValue::Mat4(mat4)) => {
                // Mat4::from_ax
                mat4.rotate(Quat::from_axis_angle(anim.axis.into(), anim.speed*info.seconds_since_update));
            },
            (DataUpdater::FloatSpeed(speed), UiValue::Float(data)) => {
                data.value += speed*info.seconds_since_update;
            },
            _ => {}
        }
    }
}

impl Widget for &mut DataUpdater {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        match self {
            DataUpdater::FloatSpeed(f32_speed) => {
                ui.add(egui::Slider::new(f32_speed, -1.0..=1.0))
            },
            DataUpdater::Rotation(rotation_animation) => {
                rotation_animation.ui(ui)
            }
        }
    }
}