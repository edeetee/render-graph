use std::{fs::{read_to_string, File}, io::Read};

use glium::{backend::Facade, uniforms::Uniforms, Surface};
use glium_utils::modular_shader::fullscreen_shader::FullscreenFrag;
use isf::Isf;
use itertools::Itertools;

use super::isf::IsfFile;

pub struct IsfShader {
    frag: FullscreenFrag
}

impl IsfShader {
    pub fn new(facade: &impl Facade, file: &IsfFile, def: &Isf) -> Self {
        // let source = read_to_string(file).unwrap();
        let mut source = generate_isf_prefix(def);
        source.push('\n');
        let mut file = File::open(&file.path).unwrap();
        file.read_to_string(&mut source).unwrap();

        Self {
            frag: FullscreenFrag::new(facade, &source)
        }
    }

    pub fn draw(&self, surface: &mut impl Surface, uniforms: impl Uniforms) {
        self.frag.draw(surface, uniforms).unwrap();
    }
}

const STANDARD_PREFIX: &'static str = r#"
#version 440

precision highp float;
precision highp int;

const int PASSINDEX = 0;
uniform vec2 res;
uniform vec2 RENDERSIZE = res;
#define RENDERSIZE res;
vec2 isf_FragNormCoord = gl_FragCoord.xy/RENDERSIZE;
"#;

fn generate_isf_prefix(def: &Isf) -> String {
    let mut prefix = String::new();

    prefix.push_str(STANDARD_PREFIX);

    for input in &def.inputs {
        let gl_ty = match input.ty {
            isf::InputType::Image => "sampler2D",
            isf::InputType::Float(_) => "float",
            isf::InputType::Point2d(_) => "vec2",
            isf::InputType::Color(_) => "vec4",
            isf::InputType::Audio(_) => "sampler2D",
            isf::InputType::AudioFft(_) => "sampler2D",
            isf::InputType::Event => "sampler2D",
            isf::InputType::Bool(_) => "bool",
            isf::InputType::Long(_) => "int",
        };
        let name = &input.name;

        prefix.push_str(&format!("uniform {gl_ty} {name};\n"));
    }

    prefix.push('\n');

    prefix
}