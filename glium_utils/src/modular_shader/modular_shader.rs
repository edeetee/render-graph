use glium::{DrawError, Surface};

///An object that can render to a surface with parameters
pub trait ModularShader {
    ///Draw to a surface
    fn draw_to<S: Surface>(&self, surface: &mut S) -> Result<(), DrawError>
    where
        Self: Sized;

    fn update(&mut self, _update: &ShaderUpdate) {}
}

pub enum ShaderUpdate {
    Resolution([f32; 2]),
}
