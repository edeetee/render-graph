use std::fs::read_to_string;

use glium::{backend::Facade, uniforms::Uniforms, Surface};
use glium_utils::modular_shader::fullscreen_shader::FullscreenFrag;
use isf::Isf;

use super::isf::IsfFile;

pub struct IsfShader {
    frag: FullscreenFrag
}

impl IsfShader {
    pub fn new(facade: &impl Facade, file: &IsfFile) -> Self {
        Self {
            frag: FullscreenFrag::new(facade, &read_to_string(file).unwrap())
        }
    }

    pub fn draw(&self, surface: &mut impl Surface, uniforms: impl Uniforms) {
        self.frag.draw(surface, uniforms);
    }
}