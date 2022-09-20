use glium::{backend::Facade, Surface};

use super::{isf_shader::IsfShader, def::NodeTypes, connection_types::ComputedInputs};


pub enum NodeShader {
    Isf(IsfShader)
}

impl NodeShader {
    pub fn new(template: &NodeTypes, facade: &impl Facade) -> Option<Self> {
        match template {
            NodeTypes::Isf{file, isf} => Some(NodeShader::Isf(IsfShader::new(facade, file, isf))),
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