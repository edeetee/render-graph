use glium::{Texture2d, GlObject};
use spout_rust::SpoutSender;


pub struct SpoutOutShader {
    spout: SpoutSender
}

impl SpoutOutShader {
    pub fn new() -> Self {
        let spout = SpoutSender::new("SpoutOut");
        Self {
            spout
        }
    }

    pub fn send(&mut self, texture: &Texture2d) {
        self.spout.send_texture(gl::TEXTURE_2D, texture.get_id(), texture.width(), texture.height());
    }
}