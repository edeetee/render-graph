use glium::{Surface, uniform, backend::Facade, framebuffer::SimpleFrameBuffer, uniforms::EmptyUniforms};
use super::{modular_shader::ModularShader, fullscreen_shader::FullscreenFrag};

pub struct UvView {
    fullscreen: FullscreenFrag
}

impl ModularShader for UvView {
    fn draw_to(&self, surface: &mut SimpleFrameBuffer<'_>) -> Result<(), glium::DrawError>
    {
            self.fullscreen.draw(
            surface, 
            EmptyUniforms
        )
    }
}

impl UvView{
    pub fn new(facade: &impl Facade) -> Self {
        Self{
            fullscreen: FullscreenFrag::new(facade,include_str!("uv.frag"))
        }
    }

    pub fn debug_listen_shader(&mut self) {
        todo!("Implement a program to listen to a shader file in debug and update automatically!!")
        //probably do this as a trait
    }
}