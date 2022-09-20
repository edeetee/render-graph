use std::{
    borrow::{Borrow, BorrowMut},
    rc::Rc,
};

use super::{def::{ComputedNodeInput, NodeTypes}, isf_shader::IsfShader, connection_types::ComputedInputs};
use egui::TextureId;
use egui_glium::EguiGlium;
use glium::{
    backend::Facade,
    framebuffer::{RenderBuffer, SimpleFrameBuffer},
    texture::SrgbTexture2d,
    Surface, Texture2d,
};
use glium_utils::modular_shader::{
    sdf::SdfView,
    uv::{UvData, UvView}, fullscreen_shader::FullscreenFrag,
};
use ouroboros::self_referencing;

#[self_referencing]
struct NodeShaderData {
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

pub struct NodeShader {
    data: NodeShaderData,
}

const DEFAULT_RES: [u32; 2] = [512, 512];

impl NodeShader {
    pub fn new(
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

        Self {
            data: NodeShaderDataBuilder {
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
            .build(),
        }
    }

    pub fn render<'a, 'b>(
        &mut self,
        target: &mut impl Surface,
        f: &mut impl Fn(&mut impl Surface),
    ) {
        let filter = glium::uniforms::MagnifySamplerFilter::Nearest;

        self.data.with_render_fb_mut(|fb| {
            fb.clear_color(0., 0., 0., 0.);
            self.shader.draw(fb, inputs);
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
