use glium::{backend::Facade, Surface, ProgramCreationError, Texture2d, uniforms::{UniformValue, Uniforms}};

use super::{node_types::NodeTypes, spout_out_shader::SpoutOutShader};
use crate::isf::shader::{IsfShader, IsfShaderLoadError};

pub enum NodeShader {
    Isf(IsfShader),
    SpoutOut(SpoutOutShader)
}

impl NodeShader {
    pub fn new(template: &NodeTypes, facade: &impl Facade) -> Option<Result<Self, IsfShaderLoadError>> {
        match template {
            NodeTypes::Isf{file, isf} => {
                Some(IsfShader::new(facade, file, isf).map(NodeShader::Isf))
            },
            NodeTypes::SpoutOut => {
                Some(Ok(NodeShader::SpoutOut(SpoutOutShader::new())))
            },
            _ => None,
        }
    }

    pub fn draw<'a, 'b>(
        &mut self,
        texture: &Texture2d,
        inputs: &ComputedInputs<'a>,
    ) {
        match self {
            NodeShader::Isf(isf) => {
                isf.draw(&mut texture.as_surface(), inputs);
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

pub struct ComputedInputs<'a> {
    vec: Vec<(&'a str, UniformValue<'a>)>,
}

impl ComputedInputs<'_> {
    pub fn first_texture(&self) -> Option<&Texture2d> {
        self.vec.iter().filter_map(|(_,tex)| {
            match *tex {
                UniformValue::Texture2d(tex, _) => Some(tex),
                _ => None,
            }
        }).next()
    }
}

impl<'a> Uniforms for ComputedInputs<'a>{
    fn visit_values<'b, F: FnMut(&str, UniformValue<'b>)>(&'b self, mut output: F) {
        for (name, input) in self.vec.iter() {
            output(name, *input);
        }
    }
}

impl<'a> FromIterator<(&'a str, UniformValue<'a>)> for ComputedInputs<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (&'a str, UniformValue<'a>)>,
    {
        ComputedInputs {
            vec: iter.into_iter().collect(),
        }
    }
}