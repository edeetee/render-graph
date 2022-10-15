use std::rc::Rc;

use glium::{backend::Facade, Surface, Texture2d, uniforms::{UniformValue, Uniforms, AsUniformValue}};

use super::{node_types::NodeType, spout_out_shader::SpoutOutShader, graph::{ProcessedInputs}};
use crate::{isf::shader::{IsfShader, IsfShaderLoadError}, obj_shader::ObjRenderer};

pub enum NodeShader {
    Isf(IsfShader),
    SpoutOut(SpoutOutShader),
    Obj(ObjRenderer)
}

impl NodeShader {
    pub fn new(template: &NodeType, facade: &impl Facade) -> Option<Result<Self, IsfShaderLoadError>> {
        match template {
            NodeType::Isf{info} => {
                Some(IsfShader::new(facade, info).map(NodeShader::Isf))
            },
            NodeType::SpoutOut => {
                Some(Ok(NodeShader::SpoutOut(SpoutOutShader::new())))
            },
            NodeType::ObjRender => {
                Some(Ok(NodeShader::Obj(ObjRenderer::new(facade).unwrap())))
            },
        }
    }

    pub fn render<'a, 'b>(
        &mut self,
        texture: &Texture2d,
        inputs: ShaderInputs<'a>,
    ) {
        match self {
            NodeShader::Isf(isf) => {
                isf.draw(&mut texture.as_surface(), &inputs);
            }
            NodeShader::Obj(obj) => {
                obj.draw(&mut texture.as_surface()).unwrap();
            }
            NodeShader::SpoutOut(spout_out) => {
                //only send if input exists
                if let Some(in_tex) = inputs.first_texture() {
                    in_tex.as_surface().fill(&texture.as_surface(), glium::uniforms::MagnifySamplerFilter::Nearest);
                    spout_out.send(texture);
                }
            }
        };
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