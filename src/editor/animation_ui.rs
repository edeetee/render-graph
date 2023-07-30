use egui::{Widget, Ui};
use glam::Vec3;

use crate::{common::animation::{DataUpdater, RotationAnimation}, widgets::limited_ui::{horizontal_drags, UiLimit}};

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

impl Widget for &mut RotationAnimation {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.label("axis");
            let axis_resp = horizontal_drags(ui, &["x", "y", "z"], UiLimit::None, &mut self.axis);

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