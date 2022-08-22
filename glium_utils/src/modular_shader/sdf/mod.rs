use super::fullscreen_shader::FullscreenFrag;
use glium::{
    backend::Facade,
    framebuffer::SimpleFrameBuffer,
    uniform,
    uniforms::{AsUniformValue, Sampler, EmptyUniforms},
    Surface, program::Uniform,
};

pub struct SdfView {
    fullscreen: FullscreenFrag,
}

impl SdfView {
    pub fn draw<'a, T>(
        &self,
        surface: &mut impl Surface,
        uv_source: Option<Sampler<'a, T>>,
    ) -> Result<(), glium::DrawError>
    where
        Sampler<'a, T>: AsUniformValue,
        T: 'a
    {
        match uv_source {
            Some(uv_source) => self.fullscreen.draw(
                surface,
                uniform! {
                    uv: uv_source,
                    has_hv: true
                },
            ),
            None => self.fullscreen.draw(surface, uniform! {
                has_hv: false
            }),
        }
    }
}

impl SdfView {
    pub fn new(facade: &impl Facade) -> Self {
        Self {
            fullscreen: FullscreenFrag::new(facade, include_str!("sdf.frag")),
            // size: [512., 512.]
        }
    }

    pub fn debug_listen_shader(&mut self) {
        todo!("Implement a program to listen to a shader file in debug and update automatically!!")
        //probably do this as a trait
    }
}
