use spout_rust::SpoutSender;
use wgpu::Texture;


pub struct SpoutOutShader {
    spout: SpoutSender
}

impl SpoutOutShader {
    pub fn new() -> Self {
        let spout = SpoutSender::new("RustSpoutOut");
        Self {
            spout
        }
    }

    pub fn send(&mut self, texture: &Texture) {
        // texture.as_hal(|hal_tex| )
        self.spout.send_texture(gl::TEXTURE_2D, texture.get_id(), texture.width(), texture.height());
    }
}