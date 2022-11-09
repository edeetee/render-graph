use glium::{Texture2d, GlObject};

#[cfg(target_os="windows")]
use spout_rust::SpoutSender;


pub struct SpoutOutShader {
    #[cfg(target_os="windows")]
    spout: SpoutSender
}

impl SpoutOutShader {
    pub fn new() -> Self {
        #[cfg(target_os="windows")]
        let spout = SpoutSender::new("RustSpoutOut");
        Self {
            #[cfg(target_os="windows")]
            spout
        }
    }

    pub fn send(&mut self, texture: &Texture2d) {
        #[cfg(target_os="windows")]
        self.spout.send_texture(gl::TEXTURE_2D, texture.get_id(), texture.width(), texture.height());
    }
}