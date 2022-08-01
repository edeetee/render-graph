use glium::{Surface, uniform, backend::Facade};
use super::{modular_shader::ModularShader, fullscreen_shader::FullscreenFrag};

pub struct SdfView {
    fullscreen: FullscreenFrag,
    size: [f32; 2]
}

impl<S: Surface> ModularShader<S> for SdfView {
    fn draw_to(&self, surface: &mut S) -> Result<(), glium::DrawError>
    {
        self.fullscreen.draw(
            surface, 
            &uniform! {
                size: self.size,
            }
        )
    }
}

impl SdfView{
    pub fn new(facade: &impl Facade) -> Self {
        Self{
            fullscreen: FullscreenFrag::new(facade,include_str!("sdf.frag")),
            size: [0., 0.]
        }
    }

    pub fn debug_listen_shader(&mut self) {
        todo!("Implement a program to listen to a shader file in debug and update automatically!!")
        //probably do this as a trait
    }
}