use glium::{backend::Facade, Surface, ProgramCreationError, Texture2d};

use super::{isf_shader::IsfShader, connection_types::ComputedInputs, node_types::NodeTypes, spout_out_shader::SpoutOutShader};


pub enum NodeShader {
    Isf(IsfShader),
    SpoutOut(SpoutOutShader)
}

impl NodeShader {
    pub fn new(template: &NodeTypes, facade: &impl Facade) -> Option<Result<Self, ProgramCreationError>> {
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
                if let Some(in_tex) = inputs.first_texture() {
                    in_tex.as_surface().fill(&mut texture.as_surface(), glium::uniforms::MagnifySamplerFilter::Nearest);
                    spout_out.send(texture);
                }
            }
        };
    }
}