use std::rc::Rc;
use egui::{TextureId};
use egui_glium::EguiGlium;
use glium::Surface;
use glium::backend::Facade;
use glium::texture::SrgbTexture2d;
use glium::framebuffer::SimpleFrameBuffer;
use glutin::surface::AsRawSurface;
use ouroboros::self_referencing;
use crate::textures::new_texture_srgb_2d;

pub struct UiTexture {
    tex: Rc<SrgbTexture2d>,
    id: TextureId,
}

impl UiTexture {
    pub fn new(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
        size: (u32, u32)
    ) -> Self {
        let tex = Rc::new(new_texture_srgb_2d(facade, size).unwrap());

        let id = egui_glium
            .painter
            .register_native_texture(tex.clone());

        Self {
            id,
            tex
        }
    }

    pub fn update_size(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium, size: (u32, u32)) {
        //we need to completely replace the texture instead of just updating it
        if self.tex.dimensions() != size {
            // let new_self = Self::new(facade, egui_glium, size);

            println!("Updating texture size from {:?} to {:?}", self.tex.dimensions(), size);

            let tex = Rc::new(new_texture_srgb_2d(facade, size).unwrap());
            egui_glium.painter.replace_native_texture(self.id, tex.clone());
            self.tex = tex;
        }
    }

    pub fn copy_from(&mut self, facade: &impl Facade, surface: &impl Surface){
        let filter = glium::uniforms::MagnifySamplerFilter::Linear;

        surface.fill(
            &SimpleFrameBuffer::new(facade, self.tex.as_ref()).unwrap(),
            filter,
        );
    }

    pub fn framebuffer(&self, facade: &impl Facade) -> Result<SimpleFrameBuffer<'_>, glium::framebuffer::ValidationError> {
        SimpleFrameBuffer::new(facade, self.tex.as_ref())
    }

    pub fn size(&self) -> (u32, u32) {
        self.tex.dimensions()
    }

    pub fn id(&self) -> TextureId {
        self.id
    }
}