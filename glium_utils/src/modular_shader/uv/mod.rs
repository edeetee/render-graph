use glium::{uniform, backend::Facade, framebuffer::SimpleFrameBuffer, Surface};
use super::{fullscreen_shader::FullscreenFrag};

pub struct UvView {
    fullscreen: FullscreenFrag,
}

pub struct UvData {
    pub scale: [f32; 2],
    pub centered: bool
}

impl UvView {
    pub fn draw(&self, surface: &mut impl Surface, scale: &[f32; 2], centered: &bool) -> Result<(), glium::DrawError>
    {
        self.fullscreen.draw(
            surface, 
            uniform! {
                scale: *scale,
                centered: *centered
            }
        )
    }
}

impl UvView{
    pub fn new(facade: &impl Facade) -> Self {
        Self{
            fullscreen: FullscreenFrag::new(facade,include_str!("uv.frag")),
            // scale: [1.,1.],
            // centered: false
        }
    }

    pub fn debug_listen_shader(&mut self) {
        todo!("Implement a program to listen to a shader file in debug and update automatically!!")
        //probably do this as a trait
    }
}