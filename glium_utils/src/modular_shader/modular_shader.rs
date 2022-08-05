use glium::{DrawError, framebuffer::SimpleFrameBuffer};

///An object that can render to a surface with parameters
pub trait ModularShader
{
    ///Draw to a surface
    fn draw_to(&self, surface: &mut SimpleFrameBuffer<'_>) -> Result<(), DrawError>;
    fn update(&mut self, _update: &ShaderUpdate) {}
}

pub enum ShaderUpdate {
    Resolution([f32; 2]),
}
