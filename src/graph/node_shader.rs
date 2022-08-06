use std::{rc::Rc, borrow::BorrowMut};

use egui::TextureId;
use egui_glium::EguiGlium;
use glium::{texture::SrgbTexture2d, framebuffer::{SimpleFrameBuffer}, backend::Facade, Surface};
use glium_utils::modular_shader::{modular_shader::{ModularShader}, sdf::SdfView, uv::UvView};
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

const DEFAULT_RES: [u32; 2] = [512, 512];

impl NodeShader {
    pub fn new(facade: &impl Facade, egui_glium: &mut EguiGlium, template: NodeTypes) -> Self{
        let tex = SrgbTexture2d::empty_with_format(
            facade, 
            glium::texture::SrgbFormat::U8U8U8U8, 
            glium::texture::MipmapsOption::NoMipmap, 
            DEFAULT_RES[0].into(), 
            DEFAULT_RES[1].into()
        ).unwrap();

        // tex.write(rect, data)
        // tex.first_layer().main_level().get_texture().
    
        let tex_rc = Rc::new(tex);
        let output_texture_egui = egui_glium.painter.register_native_texture(tex_rc.clone());
    
        let modular_shader: Option<Box<dyn ModularShader>> = match template {
            NodeTypes::Sdf => Some(Box::new(SdfView::new(facade))),
            NodeTypes::UV => Some(Box::new(UvView::new(facade))),
            _ => None
        };
    
        Self {
            modular_shader,
            data: NodeShaderDataBuilder {
                tex_id: output_texture_egui,
                tex_rc,
                tex_fb_builder: |tex_rc: &Rc<SrgbTexture2d>| {
                   let mut fb =  SimpleFrameBuffer::new(facade, tex_rc.as_ref()).unwrap();
                   fb.clear_color(0., 0., 0., 0.);
                   fb
                },
            }.build()
        }
    }

    pub fn render(&mut self, target: &mut SimpleFrameBuffer) {
        // self.with_tex_fb_mut(user)
        let filter = glium::uniforms::MagnifySamplerFilter::Nearest;

        self.data.with_tex_fb_mut(|tex_fb| {
            //fill background from prev
            // target.fill(tex_fb, filter);
            let draw_surface = target;
            let copy_target = tex_fb;

            draw_surface.clear_color_and_depth((0., 0., 0., 1.), 0.);

            // draw_surface

            //do the draw
            if let Some(shader) = self.modular_shader.as_deref() {
                shader.draw_to(draw_surface).unwrap();
            }

            //copy back to target
            draw_surface.fill(copy_target, filter);
        })
    }

    pub fn clone_tex_id(&self) -> TextureId{
        self.data.borrow_tex_id().clone()
    }
}