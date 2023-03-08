use std::rc::Rc;
use egui::TextureId;
use egui_glium::EguiGlium;
use glium::Surface;
use glium::backend::Facade;
use glium::texture::SrgbTexture2d;
use glium::framebuffer::SimpleFrameBuffer;
use ouroboros::self_referencing;
use crate::textures::new_texture_srgb_2d;

#[self_referencing]
struct ScreenTexture {
    tex: Rc<SrgbTexture2d>,
    id: TextureId,

    #[borrows(tex)]
    #[covariant]
    fb: SimpleFrameBuffer<'this>
}


impl ScreenTexture {
    pub fn generate(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
        size: (u32, u32),
    ) -> Self {

        let tex = Rc::new(new_texture_srgb_2d(facade, size).unwrap());

        let id = egui_glium
            .painter
            .register_native_texture(tex.clone());


        ScreenTextureBuilder {
            id,
            tex,
            fb_builder: |tex: &Rc<SrgbTexture2d>| {
                SimpleFrameBuffer::new(facade, tex.as_ref()).unwrap()
            },
        }
        .build()
    }
}

pub struct UiTexture {
    screen: ScreenTexture,
}

impl UiTexture {
    pub fn new(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
        size: (u32, u32)
    ) -> Self {

        Self {
            screen: ScreenTexture::generate(facade, egui_glium, size),
        }
    }

    pub fn update_size(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium, size: (u32, u32)) {
        //we need to completely replace the texture instead of just updating it
        if self.screen.borrow_tex().dimensions() != size {
            let new_screen = ScreenTexture::generate(facade, egui_glium, size);

            println!("Updating texture size from {:?} to {:?}", self.screen.borrow_tex().dimensions(), size);
        
            egui_glium.painter.replace_native_texture(*self.screen.borrow_id(), new_screen.borrow_tex().clone());

            self.screen = new_screen;
        }
    }

    pub fn copy_from(&mut self, surface: &impl Surface){
        let filter = glium::uniforms::MagnifySamplerFilter::Linear;

        // SimpleFrameBuffer 

        surface.fill(
            self.screen.borrow_fb(),
            filter,
        );
    }

    pub fn size(&self) -> (u32, u32) {
        self.screen.borrow_tex().dimensions()
    }

    pub fn clone_screen_tex_id(&self) -> TextureId {
        self.screen.borrow_id().clone()
    }
}
