use glium::{backend::Facade, Surface, ProgramCreationError};

use super::{isf_shader::IsfShader, def::NodeTypes, connection_types::ComputedInputs};


pub enum NodeShader {
    Isf(IsfShader)
}

impl NodeShader {
    pub fn new(template: &NodeTypes, facade: &impl Facade) -> Option<Result<Self, ProgramCreationError>> {
        match template {
            NodeTypes::Isf{file, isf} => {
                Some(IsfShader::new(facade, file, isf).map(NodeShader::Isf))
            },
            _ => None,
        }
    }

    pub fn draw<'a, 'b>(
        &self,
        surface: &mut impl Surface,
        inputs: &ComputedInputs<'a>,
    ) {
        match self {
            NodeShader::Isf(isf) => {
                isf.draw(surface, inputs);
            }
        };
    }
}