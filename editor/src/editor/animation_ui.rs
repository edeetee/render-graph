use egui::{Ui, Widget};
use glam::Vec3;

use crate::widgets::limited_ui::{horizontal_drags, UiLimit};
use graph::animation::{DataUpdater, RotationAnimation};

pub fn draw_dataupdater(this: &mut DataUpdater, ui: &mut Ui) -> egui::Response {
    match this {
        DataUpdater::FloatSpeed(f32_speed) => ui.add(egui::Slider::new(f32_speed, -1.0..=1.0)),
        DataUpdater::Rotation(rotation_animation) => draw_rotationanimation(rotation_animation, ui),
    }
}

pub fn draw_rotationanimation(this: &mut RotationAnimation, ui: &mut Ui) -> egui::Response {
    ui.vertical(|ui| {
        ui.label("axis");
        let axis_resp = horizontal_drags(ui, &["x", "y", "z"], UiLimit::None, &mut this.axis);

        if axis_resp.inner {
            Vec3::from_slice(&this.axis)
                .try_normalize()
                .unwrap_or(Vec3::Y)
                .write_to_slice(&mut this.axis);
        }

        ui.label("speed");
        ui.add(egui::Slider::new(&mut this.speed, -1.0..=1.0));
    })
    .response
}
