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
struct NodeShaderData{
    tex_rc: Rc<SrgbTexture2d>,
    tex_id: TextureId,

    #[borrows(tex_rc)]
    #[covariant]
    tex_fb: SimpleFrameBuffer<'this>
}

pub struct NodeShader {
    data: NodeShaderData,
    modular_shader: Option<Box<dyn ModularShader>>,
}

const DEFAULT_RES: [u32; 2] = [1920, 1080];

impl NodeShader {
    pub fn new(facade: &impl Facade, egui_glium: &mut EguiGlium, template: NodeTypes) -> Self{
        let tex = SrgbTexture2d::empty_with_format(
            facade, 
            glium::texture::SrgbFormat::U8U8U8U8, 
            glium::texture::MipmapsOption::NoMipmap, 
            DEFAULT_RES[0].into(), 
            DEFAULT_RES[1].into()
        ).unwrap();
    
        let tex_rc = Rc::new(tex);
        let output_texture_egui = egui_glium.painter.register_native_texture(tex_rc.clone());
    
        let modular_shader = match template {
            NodeTypes::Sdf => Some(Box::new(SdfView::new(facade)) as Box<dyn ModularShader>),
            _ => None
        };
    
        Self {
            modular_shader,
            data: NodeShaderDataBuilder {
                tex_id: output_texture_egui,
                tex_rc,
                tex_fb_builder: |tex_rc: &Rc<SrgbTexture2d>| SimpleFrameBuffer::new(facade, tex_rc.as_ref()).unwrap(),
            }.build()
        }
    }

    pub fn render(&mut self) {
        // self.with_tex_fb_mut(user)
        if let Some(shader) = self.modular_shader.as_deref() {
            self.data.with_tex_fb_mut(|tex_fb| {
                shader.draw_to(tex_fb).unwrap();
            })
        }

        //fill op image even if no operation
        // surface.fill(self.borrow_tex_fb(), glium::uniforms::MagnifySamplerFilter::Linear);
    }

    pub fn clone_tex_id(&self) -> TextureId{
        self.data.borrow_tex_id().clone()
    }
}