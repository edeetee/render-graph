use glam::{Mat4, Vec3, EulerRot, Quat};
use serde::{Serialize, Deserialize};

///Transformation data with helper data for human editing
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Mat4UiData {
    pub mat: Mat4,

    ///Rotation in degrees
    pub rotation: [f32; 3],
    pub scale: f32,
    pub translation: [f32; 3]
}

const EULER_ORDER: EulerRot = EulerRot::ZXY;

impl From<Mat4> for Mat4UiData {
    fn from(value: Mat4) -> Self {
        let decomposed = value.to_scale_rotation_translation();
        let rot_tuple = decomposed.1.to_euler(EULER_ORDER);
        Self {
            scale: decomposed.0.length_squared()/3.0,
            rotation: [rot_tuple.0.to_degrees(), rot_tuple.1.to_degrees(), rot_tuple.2.to_degrees()],
            translation: decomposed.2.to_array(),
            mat: value,
        }
    }
}

impl Mat4UiData {
    pub fn new_view() -> Self {
        let mut new = Self {
            translation: [0.0, 0.0, -5.0],
            mat: Mat4::IDENTITY,
            scale: 1.0,
            rotation: Default::default()
        };

        new.update_mat();

        new
    }

    pub fn quat(&self) -> Quat {
        Quat::from_euler(EULER_ORDER, 
            self.rotation[2].to_radians(), 
            self.rotation[0].to_radians(), 
            self.rotation[1].to_radians()
        )
    }

    ///Rotate by some amount and apply it to the data
    pub fn rotate(&mut self, rotation: Quat) {
        // self.rotation = tuple_to_array(self.mat.to_scale_rotation_translation().1.to_euler(EULER_ORDER));
        let delta_rot = rotation.to_euler(EULER_ORDER);
        // delta_rot.iter
        // self.rotation.iter_mut()
        self.rotation[0] += delta_rot.0.to_degrees();
        self.rotation[1] += delta_rot.1.to_degrees();
        self.rotation[2] += delta_rot.2.to_degrees();
        // self.rotation += [, delta_rot.1.to_degrees(), delta_rot.2.to_degrees()];
        
        // self.mat = Mat4::IDENTITY
        //     * Mat4::from_quat(rotation)
        //     * self.rotation_matrix()
        //     * Mat4::from_translation(Vec3::from_array(self.translation))
        //     * Mat4::from_scale(Vec3::new(self.scale, self.scale, self.scale));

        // MAt4::
        // let start_quat = Quat::from_mat4(&self.mat);

        // let temp_tuple = 
        self.update_mat();
    }

    ///Called to update the actual matrix value
    pub fn update_mat(&mut self) {
        self.mat = Mat4::IDENTITY
            * Mat4::from_quat(self.quat())
            * Mat4::from_translation(Vec3::from_array(self.translation))
            * Mat4::from_scale(Vec3::new(self.scale, self.scale, self.scale))
    }
}