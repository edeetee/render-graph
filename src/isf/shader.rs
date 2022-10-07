use std::{fs::{File, read_to_string}, io::Read, rc::Rc, time::Instant};

use glium::{backend::Facade, ProgramCreationError::{self, LinkingError}, Surface, Texture2d, uniforms::{AsUniformValue, Uniforms, UniformValue}};
use isf::{Isf, Pass};
use crate::fullscreen_shader::FullscreenFrag;
use crate::textures::new_texture_2d;

use crate::isf::meta::IsfPathInfo;

pub struct IsfShader {
    frag: FullscreenFrag,
    passes: Vec<(Pass, Texture2d)>,
    start_inst: Instant,
    prev_frame_inst: Instant,
    frame_count: u32,
}

impl IsfShader {
    pub fn new(facade: &impl Facade, path: &IsfPathInfo, def: &Isf) -> Result<Self, IsfShaderLoadError> {
        // let source = read_to_string(file).unwrap();
        let mut source = generate_isf_prefix(def);
        source.push('\n');
        let mut file = File::open(&path.path)?;
        file.read_to_string(&mut source)?;

        let passes = def.passes.iter().map(|pass| {
            (pass.clone(), new_texture_2d(facade, 1, 1).unwrap())
        })
        .collect();

        let now = Instant::now();

        // def.passes.first().unwrap().

        Ok(Self {
            frag: FullscreenFrag::new(facade, &source)?,
            start_inst: now,
            prev_frame_inst: now,
            frame_count: 0,
            passes
        })
    }

    // pub fn is_stale(&self) -> bool {
    //     let file_version = self.path.metadata().unwrap().modified().unwrap();

    //     self.version < file_version
    // }

    pub fn draw(&mut self, surface: &mut impl Surface, uniforms: &impl Uniforms) {
        let now = Instant::now();
        let time_delta = now - self.prev_frame_inst;
        let time_total = now - self.start_inst;

        let mut uniforms = IsfUniforms {
            inner: uniforms,
            time_delta: time_delta.as_secs_f32(),
            time: time_total.as_secs_f32(),
            frame_index: self.frame_count,
            pass_index: 0,
            passes: &self.passes
        };

        if self.passes.is_empty() {
            self.frag.draw(surface, &uniforms).unwrap();
        } else {
            let filter = glium::uniforms::MagnifySamplerFilter::Nearest;

            for (pass, tex) in &self.passes {
                uniforms.pass_index += 1;
                self.frag.draw(surface, &uniforms).unwrap();
                surface.fill(&tex.as_surface(), filter);
            }
        }

        self.frame_count += 1;
    }
}

struct IsfUniforms<'a, U: Uniforms> {
    frame_index: u32,
    time_delta: f32,
    time: f32,
    pass_index: u32,
    passes: &'a Vec<(Pass, Texture2d)>,
    inner: &'a U,
}

impl <U: Uniforms> Uniforms for IsfUniforms<'_, U> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        f("FRAMEINDEX", self.frame_index.as_uniform_value());
        f("TIMEDELTA", self.time_delta.as_uniform_value());
        f("TIME", self.time.as_uniform_value());
        f("PASSINDEX", self.pass_index.as_uniform_value());
        for (pass, tex) in self.passes {
            if let Some(name) = pass.target.as_ref() {
                f(name, tex.as_uniform_value());
            }
        }
        self.inner.visit_values(f);
    }
}

pub fn reload_ifs_shader(
    facade: &impl Facade,
    file: &IsfPathInfo,
) -> Result<(Isf, IsfShader), IsfShaderLoadError> {
    let isf = isf::parse(&read_to_string(&file.path)?)?;
    let shader = IsfShader::new(facade, file, &isf)?;

    Ok((isf, shader))
}


const STANDARD_PREFIX: &'static str = include_str!("prefix.glsl");

fn generate_isf_prefix(def: &Isf) -> String {
    let mut prefix = String::new();

    prefix.push_str(STANDARD_PREFIX);

    let inputs = def.inputs.iter().map(|input| {
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
        (name, gl_ty)
    });

    let passes = def.passes.iter().filter_map(|pass| {
        pass.target.as_ref().map(|name| (name, "sampler2D"))
    });

    for (name, gl_ty) in inputs.chain(passes) {
        prefix.push_str(&format!("uniform {gl_ty} {name};\n"));
    }

    prefix.push('\n');

    prefix
}

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
                        write!(f, "CompilationError for {shader_type:?} (\n{source})")
                    }
                    LinkingError(source) => {
                        write!(f, "LinkingError (\n{source})")
                    },
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