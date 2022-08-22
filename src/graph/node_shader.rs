use std::{
    borrow::{Borrow, BorrowMut},
    rc::Rc,
};

use super::def::{NodeConnectionTypes, NodeData, NodeTypes, NodeValueTypes, ComputedNodeInput};
use egui::TextureId;
use egui_glium::EguiGlium;
use egui_node_graph::{InputParam, Node};
use glium::{backend::Facade, framebuffer::SimpleFrameBuffer, texture::SrgbTexture2d, Surface};
use glium_utils::modular_shader::{sdf::SdfView, uv::{UvView, UvData}};
use ouroboros::self_referencing;

// pub struct FrameBufferShader<'a>{
//     modular_shader: ,
// }

#[self_referencing]
struct NodeShaderData {
    tex_rc: Rc<SrgbTexture2d>,
    tex_id: TextureId,

    #[borrows(tex_rc)]
    #[covariant]
    tex_fb: SimpleFrameBuffer<'this>,
}

enum Shader {
    Sdf(SdfView),
    Uv(UvView),
}

// impl<'a, T> From<&'a Shader> for &'a dyn ModularShader<T> {
//     fn from(en: &'a Shader) -> Self {
//         match en {
//             Shader::Sdf(sdf) => sdf,
//             Shader::Uv(uv) => uv,
//         }
//     }
// }

impl Shader {
    fn new(template: NodeTypes, facade: &impl Facade) -> Option<Self> {
        match template {
            NodeTypes::Sdf => Some(Shader::Sdf(SdfView::new(facade))),
            NodeTypes::Uv => Some(Shader::Uv(UvView::new(facade))),
            _ => None,
        }
    }

    // fn as_modular_shader<T>(&self) -> &dyn ModularShader<T> {
    //     self.into()
    // }

    fn draw<'a, 'b>(
        &self,
        surface: &mut impl Surface,
        named_inputs: impl Iterator<
            Item = (&'a str, ComputedNodeInput<'b>),
        >,
    ) {
        match self {
            Shader::Sdf(sdf) => {
                let mut source_uv = None;

                for input in named_inputs {
                    match input {
                        ("uv", ComputedNodeInput::Texture(tex)) => {
                            source_uv = Some(tex);
                        }
                        _ => {}
                    }
                }

                let source_uv_sample = match &source_uv {
                    Some(source_uv) => Some(source_uv.sampled()),
                    None => None,
                };

                sdf.draw(surface, source_uv_sample);
            },
            Shader::Uv(uv) => {
                let mut scale = &[1., 1.];
                let mut centered = &false;

                for input in named_inputs {
                    match input {
                        ("scale", ComputedNodeInput::Vec2(val)) => {
                            scale = val;
                        },
                        ("centered", ComputedNodeInput::Bool(val)) => {
                            centered = val;
                        },
                        _ => {}
                    }
                }

                uv.draw(surface, scale, centered);
            },
        };
    }
}

pub struct NodeShader {
    data: NodeShaderData,
    shader: Shader,
}

const DEFAULT_RES: [u32; 2] = [512, 512];

impl NodeShader {
    pub fn new(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
        template: NodeTypes,
    ) -> Option<Self> {
        let tex = SrgbTexture2d::empty_with_format(
            facade,
            glium::texture::SrgbFormat::U8U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            DEFAULT_RES[0].into(),
            DEFAULT_RES[1].into(),
        )
        .unwrap();

        // tex.write(rect, data)
        // tex.first_layer().main_level().get_texture().

        let tex_rc = Rc::new(tex);
        let output_texture_egui = egui_glium.painter.register_native_texture(tex_rc.clone());

        let shader = Shader::new(template, facade)?;

        Some(Self {
            shader,
            data: NodeShaderDataBuilder {
                tex_id: output_texture_egui,
                tex_rc,
                tex_fb_builder: |tex_rc: &Rc<SrgbTexture2d>| {
                    let mut fb = SimpleFrameBuffer::new(facade, tex_rc.as_ref()).unwrap();
                    fb.clear_color(0., 0., 0., 0.);
                    fb
                },
            }
            .build(),
        })
    }

    pub fn render<'a, 'b>(&mut self, target: &mut SimpleFrameBuffer, named_inputs: impl Iterator<
        Item = (&'a str, ComputedNodeInput<'b>)>) {
        // self.with_tex_fb_mut(user)
        let filter = glium::uniforms::MagnifySamplerFilter::Nearest;

        self.data.with_tex_fb_mut(|tex_fb| {
            //fill background from prev
            // target.fill(tex_fb, filter);
            let draw_surface = target;
            let copy_target = tex_fb;

            draw_surface.clear_color_and_depth((0., 0., 0., 1.), 0.);

            // draw_surface
            self.shader.draw(draw_surface, named_inputs);
                // .as_modular_shader()
                // .draw_to(draw_surface)
                // .unwrap();

            //copy back to target
            draw_surface.fill(copy_target, filter);
        })
    }

    pub fn tex_rc(&self) -> Rc<SrgbTexture2d> {
        self.data.borrow_tex_rc().to_owned()
    }

    pub fn clone_tex_id(&self) -> TextureId {
        self.data.borrow_tex_id().clone()
    }
}
