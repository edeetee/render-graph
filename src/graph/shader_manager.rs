use std::{rc::Rc, borrow::BorrowMut};

use egui::TextureId;
use egui_glium::EguiGlium;
use glium::{texture::SrgbTexture2d, framebuffer::{SimpleFrameBuffer}, backend::Facade, Surface};
use glium_utils::modular_shader::{modular_shader::{ModularShader, ModularFrameBuffer}, sdf::SdfView};
use super::def::{NodeTypes};
use ouroboros::self_referencing;

// pub struct FrameBufferShader<'a>{
//     modular_shader: ,
// }

#[self_referencing]
pub struct ShaderData {
    tex_rc: Rc<SrgbTexture2d>,
    tex_id: TextureId,

    // #[borrows(tex_rc)]
    // #[not_covariant]
    modular_shader: Option<Box<dyn ModularFrameBuffer>>,

    #[borrows(tex_rc)]
    #[covariant]
    // #[covariant]
    tex_fb: SimpleFrameBuffer<'this>
}

const DEFAULT_RES: [u32; 2] = [1920, 1080];

pub fn new_shader_data(facade: &impl Facade, egui_glium: &mut EguiGlium, template: NodeTypes) -> ShaderData{
    let tex = SrgbTexture2d::empty_with_format(
        facade, 
        glium::texture::SrgbFormat::U8U8U8U8, 
        glium::texture::MipmapsOption::NoMipmap, 
        DEFAULT_RES[0].into(), 
        DEFAULT_RES[1].into()
    ).unwrap();

    // let a = tex

    let tex_rc = Rc::new(tex);
    let output_texture_egui = egui_glium.painter.register_native_texture(tex_rc.clone());

    let get_modular_shader = |tex_r| match template {
        NodeTypes::Sdf => Some(Box::new(SdfView::new(facade)) as Box<dyn ModularShader<_>>),
        _ => None
    };

    ShaderDataBuilder {
        tex_id: output_texture_egui,
        modular_shader_builder: get_modular_shader,
        tex_rc,
        tex_fb_builder: |tex_rc: &Rc<SrgbTexture2d>| SimpleFrameBuffer::new(facade, tex_rc.as_ref()).unwrap(),
    }.build()
}

impl ShaderData {
    pub fn render(&mut self) {
        if let Some(shader) = self.borrow_modular_shader().as_deref() {
            shader.draw_to(&mut self.borrow_tex_fb()).unwrap();
        }

        //fill op image even if no operation
        // surface.fill(self.borrow_tex_fb(), glium::uniforms::MagnifySamplerFilter::Linear);
    }

    pub fn clone_tex_id(&self) -> TextureId{
        self.borrow_tex_id().clone()
    }
}