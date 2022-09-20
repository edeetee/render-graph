use glium::{backend::Facade, Surface};

use super::{isf_shader::IsfShader, def::NodeTypes, connection_types::ComputedInputs};



enum Shader {
    Isf(IsfShader)
}

impl Shader {
    fn new(template: &NodeTypes, facade: &impl Facade) -> Option<Self> {
        match template {
            NodeTypes::Isf{file, isf} => Some(Shader::Isf(IsfShader::new(facade, file, isf))),
            _ => None,
        }
    }

    fn draw<'a, 'b>(
        &self,
        surface: &mut impl Surface,
        inputs: &ComputedInputs<'a>,
    ) {
        match self {
            Shader::Isf(isf) => {
                isf.draw(surface, inputs);
            }
        };
    }
}