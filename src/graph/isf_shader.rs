use std::{fs::{File, read_to_string}, io::Read};

use glium::{backend::Facade, uniforms::Uniforms, Surface, ProgramCreationError};
use isf::Isf;

use super::{isf::IsfPathInfo, fullscreen_shader::FullscreenFrag, node_types::NodeTypes, shaders::NodeShader};

pub struct IsfShader {
    frag: FullscreenFrag,
    // version: SystemTime,
    // path: PathBuf
}

impl IsfShader {
    pub fn new(facade: &impl Facade, path: &IsfPathInfo, def: &Isf) -> Result<Self, ProgramCreationError> {
        // let source = read_to_string(file).unwrap();
        let mut source = generate_isf_prefix(def);
        source.push('\n');
        let mut file = File::open(&path.path).unwrap();
        file.read_to_string(&mut source).unwrap();

        Ok(Self {
            frag: FullscreenFrag::new(facade, &source)?,
            // path: path.path.clone(),
            // version: file.metadata().unwrap().modified().unwrap()
        })
    }

    // pub fn is_stale(&self) -> bool {
    //     let file_version = self.path.metadata().unwrap().modified().unwrap();

    //     self.version < file_version
    // }

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

// #[derive(Debug)]
pub enum IsfShaderLoadError {
    IoError(std::io::Error),
    ParseError(isf::ParseError),
    CompileError(glium::program::ProgramCreationError),
}

impl std::fmt::Debug for IsfShaderLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(arg0) => arg0.fmt(f),
            Self::ParseError(arg0) => arg0.fmt(f),
            Self::CompileError(arg0) => {
                match arg0 {
                    glium::ProgramCreationError::CompilationError(source, shader_type) => {
                        // f.debug_struct(&format!()).finish()
                        // write!()
                        write!(f, "CompilationError for {shader_type:?} (\n{source})")
                    }
                    _ => arg0.fmt(f),
                }
            }
        }
    }
}

impl From<std::io::Error> for IsfShaderLoadError {
    fn from(err: std::io::Error) -> Self {
        IsfShaderLoadError::IoError(err)
    }
}

impl From<glium::program::ProgramCreationError> for IsfShaderLoadError {
    fn from(err: glium::program::ProgramCreationError) -> Self {
        IsfShaderLoadError::CompileError(err)
    }
}

impl From<isf::ParseError> for IsfShaderLoadError {
    fn from(err: isf::ParseError) -> Self {
        IsfShaderLoadError::ParseError(err)
    }
}

pub fn reload_ifs_shader(
    facade: &impl Facade,
    file: IsfPathInfo,
) -> Result<(NodeTypes, NodeShader), IsfShaderLoadError> {
    let new_template = NodeTypes::Isf {
        isf: isf::parse(&read_to_string(&file.path).unwrap())?,
        file,
    };
    let new_shader = NodeShader::new(&new_template, facade).unwrap()?;

    Ok((new_template, new_shader))
}
