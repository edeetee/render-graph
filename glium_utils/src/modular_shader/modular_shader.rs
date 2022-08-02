use glium::{DrawError, framebuffer::SimpleFrameBuffer};

///An object that can render to a surface with parameters
pub trait ModularShader
{
    ///Draw to a surface
    fn draw_to(&self, surface: &mut SimpleFrameBuffer<'_>) -> Result<(), DrawError>;
    fn update(&mut self, _update: &ShaderUpdate) {}
}

pub trait ModularFrameBuffer: ModularShader
{
    fn draw_to(&self, fb: &mut SimpleFrameBuffer<'_>) -> Result<(), DrawError> {
        ModularShader::draw_to(self, fb)
    }
}

pub enum ShaderUpdate {
    Resolution([f32; 2]),
}
