use std::{rc::Rc, pin::Pin};

use egui::TextureId;
use egui_glium::EguiGlium;
use glium::{texture::SrgbTexture2d, framebuffer::{SimpleFrameBuffer, ToColorAttachment}, backend::Facade, Frame, Surface};
use glium_utils::modular_shader::{modular_shader::ModularShader, sdf::SdfView};
use palette::Srgb;
use super::def::{NodeTypes};
use ouroboros::self_referencing;


#[self_referencing]
pub struct ShaderData<S: Surface> {
    tex: SrgbTexture2d,
    tex_id: TextureId,
    modular_shader: Option<Box<dyn ModularShader<S>>>,

    #[borrows(tex)]
    #[covariant]
    // #[covariant]
    tex_fb: SimpleFrameBuffer<'this>
}

const DEFAULT_RES: [u32; 2] = [1920, 1080];

pub fn new_shader_data<S: Surface, F: Facade>(facade: &F, egui_glium: &mut EguiGlium, template: NodeTypes) -> ShaderData<S>{
    let tex = SrgbTexture2d::empty_with_format(
        facade, 
        glium::texture::SrgbFormat::U8U8U8U8, 
        glium::texture::MipmapsOption::NoMipmap, 
        DEFAULT_RES[0].into(), 
        DEFAULT_RES[1].into()
    ).unwrap();

    let tex_rc = Rc::new(tex);
    let output_texture_egui = egui_glium.painter.register_native_texture(tex_rc.clone());

    let modular_shader: Option<Box<dyn ModularShader<_>>> = match template {
        NodeTypes::Sdf => Some(Box::new(SdfView::new(facade))),
        _ => None
    };

    ShaderDataBuilder {
        tex_id: output_texture_egui,
        modular_shader,
        tex,
        tex_fb_builder: |tex: &SrgbTexture2d| SimpleFrameBuffer::new(facade, tex).unwrap(),
    }.build()
}

impl<S: Surface> ShaderData<S> {
    pub fn render(&self, surface: &mut S) {
        if let Some(shader) = self.borrow_modular_shader().as_deref() {
            shader.draw_to(surface);
            surface.fill(self.borrow_tex_fb(), glium::uniforms::MagnifySamplerFilter::Linear);
        }
    }

    pub fn clone_tex_id(&self) -> TextureId{
        self.borrow_tex_id().clone()
    }
}