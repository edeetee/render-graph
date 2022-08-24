use std::{path::Path, fs::File};

use assets_manager::AssetCache;
use glium::{uniform, backend::Facade, framebuffer::SimpleFrameBuffer, Surface};
use super::{fullscreen_shader::FullscreenFrag, modular_shader::{ASSETS, CachedFile}};

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

// const MANIFEST: &'static str = concat!(, );

impl UvView{
    pub fn new(facade: &impl Facade) -> Self {

        let shader_file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src\\modular_shader\\uv\\uv.frag");

        let mut asset = CachedFile::new(File::open(shader_file).unwrap());

        println!("{}", asset.read());

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