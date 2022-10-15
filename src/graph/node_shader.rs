use wgpu::Device;

use super::{node_types::NodeType, spout_out_shader::SpoutOutShader};
use crate::isf::shader::{IsfShader, IsfShaderLoadError};

pub enum NodeShader {
    Isf(IsfShader),
    // SpoutOut(SpoutOutShader)
}

impl NodeShader {
    pub fn new(template: &NodeType, device: &Device) -> Option<Result<Self, IsfShaderLoadError>> {
        match template {
            NodeType::Isf{info} => {
                Some(IsfShader::new(device, info).map(NodeShader::Isf))
            },
            // NodeType::SpoutOut => {
            //     Some(Ok(NodeShader::SpoutOut(SpoutOutShader::new())))
            // },
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
            // NodeShader::SpoutOut(spout_out) => {
            //     //only send if input exists
            //     if let Some(in_tex) = inputs.first_texture() {
            //         in_tex.as_surface().fill(&texture.as_surface(), glium::uniforms::MagnifySamplerFilter::Nearest);
            //         spout_out.send(texture);
            //     }
            // }
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