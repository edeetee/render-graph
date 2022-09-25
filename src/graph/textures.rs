use std::{
    rc::Rc,
};

// use super::{def::{ComputedNodeInput, NodeTypes}, shaders::Shader};
use egui::TextureId;
use egui_glium::EguiGlium;
use glium::{
    backend::Facade,
    framebuffer::{SimpleFrameBuffer},
    texture::{SrgbTexture2d, Dimensions},
    Surface, Texture2d, GlObject,
};

use ouroboros::self_referencing;

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
    ) -> Self {

        let mipmaps = glium::texture::MipmapsOption::NoMipmap;
        let format = glium::texture::SrgbFormat::U8U8U8U8;
        let size = Dimensions::Texture2d { width: DEFAULT_RES[0], height: DEFAULT_RES[1] };

        let tex = Rc::new(
            SrgbTexture2d::empty_with_format(
                facade,
                format,
                mipmaps,
                DEFAULT_RES[0].into(),
                DEFAULT_RES[1].into(),
            )
            .unwrap(),
        );

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

pub struct NodeTextures {
    screen: ScreenTexture,
    // pub gl_id: gl::types::GLuint,
    render: Rc<Texture2d>,
}

const DEFAULT_RES: [u32; 2] = [512, 512];

impl NodeTextures {
    pub fn new(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
    ) -> Self {
        let mipmaps = glium::texture::MipmapsOption::NoMipmap;
        
        let format = glium::texture::UncompressedFloatFormat::F32F32F32F32;
        // let gl_format = gl::RGBA32F;

        // let gl_id = unsafe {
        //     let mut id: gl::types::GLuint = 0;
        //     gl::GenTextures(1, &mut id as *mut u32);
        //     gl::BindTexture(gl::TEXTURE_2D, id);
        //     gl::TexStorage2D(gl::TEXTURE_2D, 0, gl_format, DEFAULT_RES[0] as gl::types::GLsizei, DEFAULT_RES[1] as gl::types::GLsizei);

        //     id
        // };

        // let render = unsafe{
        //     let size = Dimensions::Texture2d { width: DEFAULT_RES[0], height: DEFAULT_RES[1] };

        //     Rc::new(
        //         Texture2d::from_id(facade, format, gl_id, true, mipmaps, size)
        //     )
        // };

        // let id = render.as_ref().get_id();

        let render = Rc::new(
            Texture2d::empty_with_format(
                facade,
                glium::texture::UncompressedFloatFormat::F32F32F32F32,
                mipmaps,
                DEFAULT_RES[0].into(),
                DEFAULT_RES[1].into(),
            )
            .unwrap(),
        );

        // let gl_id = render.get_id();

        Self {
            screen: ScreenTexture::generate(facade, egui_glium),
            render,
            // gl_id
        }
    }

    pub fn draw<'a>(
        &mut self,
        mut draw: impl FnMut(&Texture2d),
    ) {
        let filter = glium::uniforms::MagnifySamplerFilter::Nearest;

        self.render.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

        draw(&self.render);
        self.render.as_surface().fill(
            self.screen.borrow_fb(),
            filter,
        );
    }

    pub fn tex_for_sampling(&self) -> Rc<Texture2d> {
        self.render.clone()
    }

    pub fn clone_screen_tex_id(&self) -> TextureId {
        self.screen.borrow_id().clone()
    }
}
