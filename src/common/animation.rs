use std::time::Duration;
use glam::{Vec3, Quat};
use serde::{Serialize, Deserialize};
use super::{def::UiValue};

#[derive(Debug, Serialize, Deserialize)]
pub struct RotationAnimation {
    pub axis: [f32; 3],
    pub speed: f32
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