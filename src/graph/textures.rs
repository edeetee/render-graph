use std::{
    rc::Rc,
};

// use super::{def::{ComputedNodeInput, NodeTypes}, shaders::Shader};
use egui::TextureId;
use egui_glium::EguiGlium;
use glium::{
    backend::Facade,
    framebuffer::{SimpleFrameBuffer},
    texture::SrgbTexture2d,
    Surface, Texture2d, uniforms::Uniforms,
};

use ouroboros::self_referencing;

#[self_referencing]
struct NodeTexturesInner {
    screen_tex: Rc<SrgbTexture2d>,
    screen_id: TextureId,
    #[borrows(screen_tex)]
    #[covariant]
    screen_fb: SimpleFrameBuffer<'this>,

    render_tex: Rc<Texture2d>,
    #[borrows(render_tex)]
    #[covariant]
    render_fb: SimpleFrameBuffer<'this>,
}

impl NodeTexturesInner {
    pub fn generate(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
    ) -> Self {
        let mipmaps = glium::texture::MipmapsOption::NoMipmap;

        let screen_tex = Rc::new(
            SrgbTexture2d::empty_with_format(
                facade,
                glium::texture::SrgbFormat::U8U8U8U8,
                mipmaps,
                DEFAULT_RES[0].into(),
                DEFAULT_RES[1].into(),
            )
            .unwrap(),
        );

        let render_tex = Rc::new(
            Texture2d::empty_with_format(
                facade,
                glium::texture::UncompressedFloatFormat::F32F32F32F32,
                mipmaps,
                DEFAULT_RES[0].into(),
                DEFAULT_RES[1].into(),
            )
            .unwrap(),
        );

        let screen_id = egui_glium
            .painter
            .register_native_texture(screen_tex.clone());

        NodeTexturesInnerBuilder {
            screen_id,
            screen_tex,
            screen_fb_builder: |tex: &Rc<SrgbTexture2d>| {
                SimpleFrameBuffer::new(facade, tex.as_ref()).unwrap()
            },
            render_tex,
            render_fb_builder: |tex: &Rc<Texture2d>| {
                SimpleFrameBuffer::new(facade, tex.as_ref()).unwrap()
            },
        }
        .build()
    }
}



pub struct NodeTextures {
    data: NodeTexturesInner,
    // shader: Shader,
}

const DEFAULT_RES: [u32; 2] = [512, 512];

impl NodeTextures {
    pub fn new(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
    ) -> Self {

        Self {
            // shader,
            data: NodeTexturesInner::generate(facade, egui_glium),
        }
    }

    pub fn draw<'a>(
        &mut self,
        draw: impl Fn(&mut SimpleFrameBuffer),
        // uniforms: impl Uniforms,
        // named_inputs: impl Iterator<Item = (&'a str, ComputedNodeInput)>,
    ) {
        let filter = glium::uniforms::MagnifySamplerFilter::Nearest;

        self.data.with_render_fb_mut(|fb| {
            fb.clear_color(0., 0., 0., 0.);
            draw(fb);
            // self.shader.draw(fb, uniforms);
        });

        self.data
            .borrow_render_fb()
            .fill(self.data.borrow_screen_fb(), filter);
    }

    pub fn tex_for_sampling(&self) -> Rc<Texture2d> {
        self.data.borrow_render_tex().clone()
    }

    pub fn clone_screen_tex_id(&self) -> TextureId {
        self.data.borrow_screen_id().clone()
    }
}
