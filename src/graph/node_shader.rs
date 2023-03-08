use std::rc::Rc;

use glium::{backend::Facade, Surface, texture::Texture2d, uniforms::{UniformValue, Uniforms, AsUniformValue}, framebuffer::SimpleFrameBuffer};

use super::{node_types::NodeType, spout_out_shader::SpoutOutShader, graph::{ProcessedInputs}};
use crate::{isf::shader::{IsfShader}, obj_shader::renderer::ObjRenderer, textures::{TextureManager}, gl_expression::GlExpressionRenderer};

/// Holds shaders for the fast rendering loop
pub enum NodeShader {
    Isf(IsfShader),
    SpoutOut(SpoutOutShader),
    Obj(ObjRenderer),
    Expression(GlExpressionRenderer)
}

impl NodeShader {
    pub fn new(template: &NodeType, facade: &impl Facade) -> Option<anyhow::Result<Self>> {
        match template {
            NodeType::Isf{info} => {
                Some(IsfShader::new(facade, info).map_err(anyhow::Error::new).map(NodeShader::Isf))
            },
            NodeType::SharedOut => {
                Some(Ok(NodeShader::SpoutOut(SpoutOutShader::new())))
            },
            NodeType::ObjRender => {
                Some(Ok(NodeShader::Obj(ObjRenderer::new(facade).unwrap())))
            },
            NodeType::Expression { .. } => {
                Some(Ok(NodeShader::Expression(GlExpressionRenderer::new(facade))))
            }
        }
    }

    pub fn render(
        &mut self,
        facade: &impl Facade,
        textures: &mut TextureManager,
        inputs: ShaderInputs<'_>,
    ) -> anyhow::Result<Rc<Texture2d>> {
        let color: Rc<Texture2d> = textures.get_color(facade);

        match self {
            NodeShader::Expression(renderer) => {
                let mut surface = color.as_surface();
                surface.clear_color(0.0, 0.0, 0.0, 0.0);
                renderer.draw(&mut surface, &inputs)?;
            }
            NodeShader::Isf(isf) => {
                let mut surface = color.as_surface();
                surface.clear_color(0.0, 0.0, 0.0, 0.0);
                isf.draw(&mut surface, &inputs)?;
            }
            NodeShader::Obj(obj) => {
                let depth = textures.get_depth(facade);
                let mut fb = SimpleFrameBuffer::with_depth_buffer(facade, color.as_ref(), depth.as_ref()).unwrap();
                fb.clear_color_and_depth((0.0,0.0,0.0,0.0), f32::INFINITY);
                obj.draw(&mut fb, &inputs)?;
            }
            NodeShader::SpoutOut(spout_out) => {
                //only send if input exists
                if let Some(in_tex) = inputs.first_texture() {
                    in_tex.as_surface().fill(&color.as_surface(), glium::uniforms::MagnifySamplerFilter::Nearest);
                    spout_out.send(&color);
                }
            }
        };

        Ok(color)
    }
}

pub struct ShaderInputs<'a> {
    node_inputs: &'a ProcessedInputs<'a, Rc<Texture2d>>,
}

impl ShaderInputs<'_> {
    pub fn first_texture(&self) -> Option<&Texture2d> {
        self.node_inputs.iter().filter_map(|(_,_,tex)| {
            tex.as_ref().map(Rc::as_ref)
        }).next()
    }
}

impl<'a> Uniforms for ShaderInputs<'a>{
    fn visit_values<'b, F: FnMut(&str, UniformValue<'b>)>(&'b self, mut output: F) {
        for (name, input, tex) in self.node_inputs {
            let option_val = tex.as_ref()
                .map(Rc::as_ref)
                .map(Texture2d::as_uniform_value)
                .or_else(|| input.value.as_shader_input());

            if let Some(val) = option_val {
                output(name, val);
            }
        }
    }
}

impl <'a> From<&'a ProcessedInputs<'a, Rc<Texture2d>>> for ShaderInputs<'a> {
    fn from(inputs: &'a ProcessedInputs<'a, Rc<Texture2d>>) -> Self {
        ShaderInputs { node_inputs: inputs }
    }
}