use std::rc::Rc;

use glium::{
    backend::Facade,
    framebuffer::SimpleFrameBuffer,
    texture::Texture2d,
    uniforms::{AsUniformValue, UniformType, UniformValue, Uniforms},
    Surface,
};

use super::{graph_utils::ProcessedInputs, node_types::NodeType, spout_out_shader::SpoutOutShader};
use crate::{
    gl_expression::GlExpressionRenderer, isf::shader::IsfShader, obj_shader::renderer::ObjRenderer,
    textures::TextureManager,
};

/// Holds shaders for the fast rendering loop
pub enum NodeShader {
    Isf(IsfShader),
    SpoutOut(SpoutOutShader),
    Obj(ObjRenderer),
    Expression(GlExpressionRenderer),
}

impl NodeShader {
    pub fn new(template: &NodeType, facade: &impl Facade) -> Option<anyhow::Result<Self>> {
        match template {
            NodeType::Isf { info } => Some(
                IsfShader::new(facade, info)
                    .map_err(anyhow::Error::new)
                    .map(NodeShader::Isf),
            ),
            NodeType::SharedOut => Some(Ok(NodeShader::SpoutOut(SpoutOutShader::new()))),
            NodeType::ObjRender => Some(Ok(NodeShader::Obj(ObjRenderer::new(facade).unwrap()))),
            NodeType::Expression { .. } => Some(Ok(NodeShader::Expression(
                GlExpressionRenderer::new(facade),
            ))),
        }
    }

    pub fn render(
        &mut self,
        facade: &impl Facade,
        textures: &mut TextureManager,
        inputs: impl UniformsExt,
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
                let mut fb =
                    SimpleFrameBuffer::with_depth_buffer(facade, color.as_ref(), depth.as_ref())
                        .unwrap();
                fb.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), f32::INFINITY);
                obj.draw(&mut fb, &inputs)?;
            }
            NodeShader::SpoutOut(spout_out) => {
                //only send if input exists
                if let Some(in_tex) = inputs.first_texture() {
                    in_tex.as_surface().fill(
                        &color.as_surface(),
                        glium::uniforms::MagnifySamplerFilter::Nearest,
                    );
                    spout_out.send(&color);
                }
            }
        };

        Ok(color)
    }
}

pub struct ProcessedShaderNodeInputs<'a> {
    pub node_inputs: &'a ProcessedInputs<'a, Rc<Texture2d>>,
}

impl ProcessedShaderNodeInputs<'_> {
    pub fn first_texture(&self) -> Option<&Texture2d> {
        self.node_inputs
            .iter()
            .filter_map(|(_, _, tex)| tex.as_ref().map(Rc::as_ref))
            .next()
    }
}

impl<'a> Uniforms for ProcessedShaderNodeInputs<'a> {
    fn visit_values<'b, F: FnMut(&str, UniformValue<'b>)>(&'b self, mut output: F) {
        for (name, input, tex) in self.node_inputs {
            let option_val = tex
                .as_ref()
                .map(Rc::as_ref)
                .map(Texture2d::as_uniform_value)
                .or_else(|| input.value.as_shader_input());

            if let Some(val) = option_val {
                output(name, val);
            }
        }
    }
}

pub trait UniformsExt: Uniforms {
    fn first_texture(&self) -> Option<&Texture2d> {
        let mut texture = None;

        self.visit_values(|_, value| match value {
            UniformValue::Texture2d(img, _) => {
                texture = Some(img);
            }
            _ => {}
        });

        texture
    }
}

impl<T: Uniforms> UniformsExt for T {}

impl<'a> From<&'a ProcessedInputs<'a, Rc<Texture2d>>> for ProcessedShaderNodeInputs<'a> {
    fn from(inputs: &'a ProcessedInputs<'a, Rc<Texture2d>>) -> Self {
        ProcessedShaderNodeInputs {
            node_inputs: inputs,
        }
    }
}
