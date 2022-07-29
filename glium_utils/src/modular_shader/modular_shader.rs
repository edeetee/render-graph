use glium::{DrawError, Surface, Frame};

///An object that can render to a surface with parameters
pub trait ModularShader<S: Surface>
{
    ///Draw to a surface
    fn draw_to(&self, surface: &mut S) -> Result<(), DrawError>;

    fn update(&mut self, _update: &ShaderUpdate) {}
}

pub enum ShaderUpdate {
    Resolution([f32; 2]),
}
