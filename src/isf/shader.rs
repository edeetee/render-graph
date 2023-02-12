use std::{fs::{File}, io::Read, time::Instant, num::ParseIntError, str::FromStr};

use glium::{backend::Facade, Surface, Texture2d, uniforms::{AsUniformValue, Uniforms, UniformValue}, DrawError, program::Uniform};
use isf::{Isf, Pass};

use strum::Display;
use thiserror::Error;
use crate::{fullscreen_shader::FullscreenFrag, util::GlProgramCreationError, textures::DEFAULT_RES};
use crate::textures::new_texture_2d;

use super::meta::{IsfInfo};

pub struct IsfShader {
    frag: FullscreenFrag,
    passes: Vec<PassTexture>,
    res: (u32, u32),
    start_inst: Instant,
    prev_frame_inst: Instant,
    frame_count: u32,
}

struct PassTexture{
    pass: Pass,
    texture: Texture2d
}

impl Uniforms for PassTexture {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        if let Some(name) = self.pass.target.as_ref() {
            f(name, self.texture.as_uniform_value());
        }
    }
}

#[derive(Error, Debug)]
#[error("Could not parse pass size {0}")]
pub struct PassParseError(String);

fn default_or_parse<T: FromStr>(default: T, text: &Option<String>) -> Result<T, PassParseError> {
    match text {
        Some(text) => text.parse().map_err(|_| PassParseError(text.clone())),
        None => Ok(default),
    }
}

fn calculate_pass_size(pass: &Pass, (width, height): (u32, u32)) -> Result<(u32, u32), PassParseError> {
    Ok((
        default_or_parse(width, &pass.width)?,
        default_or_parse(height, &pass.height)?,
    ))
}

impl PassTexture {
    fn new(facade: &impl Facade, pass: Pass, res: (u32, u32)) -> Result<PassTexture, PassParseError> {
        let size = calculate_pass_size(&pass, res)?;

        Ok(Self {
            pass,
            texture: new_texture_2d(facade, size).unwrap()
        })
    }

    pub fn update_size(&mut self, facade: &impl Facade, size: (u32, u32)) {
        //allow unwrap here as you cannot make a valid PassTexture with an unparsable pass size
        let size = calculate_pass_size(&self.pass, size).unwrap();
        self.texture = new_texture_2d(facade, size).unwrap()
    }
}

use meval::{Expr, Context};

impl IsfShader {
    pub fn new(facade: &impl Facade, isf: &IsfInfo) -> Result<Self, IsfShaderLoadError> {
        // let source = read_to_string(file).unwrap();
        let mut source = generate_isf_prefix(&isf.def);
        source.push('\n');
        let mut file = File::open(&isf.path)?;
        file.read_to_string(&mut source)?;

        source = source
            .replace("gl_FragColor", "isf_FragColor")
            .replace("varying", "out");

        let res = DEFAULT_RES;

        let passes = isf.def.passes.iter()
            .map(|pass| {
                PassTexture::new(facade, pass.clone(), res)
            })
            .collect::<Result<_,_>>()?;

        let now = Instant::now();

        // def.passes.first().unwrap().

        Ok(Self {
            frag: FullscreenFrag::new(facade, &source)?,
            start_inst: now,
            prev_frame_inst: now,
            frame_count: 0,
            passes,
            res
        })
    }

    pub fn draw(&mut self, surface: &mut impl Surface, uniforms: &impl Uniforms) -> Result<(), DrawError> {
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
            self.frag.draw(surface, &uniforms)?;
        } else {
            //TODO: Update dimensions of surfaces
            // let dimens = surface.get_dimensions();
            // if dimens != self.res {
            //     self.res = dimens;
            //     for pass in &self.passes {
            //         // pass.update_size(surface, dimens)
            //     }
            // }

            let filter = glium::uniforms::MagnifySamplerFilter::Nearest;

            for pass_tex in &self.passes {
                uniforms.pass_index += 1;
                self.frag.draw(surface, &uniforms)?;
                surface.fill(&pass_tex.texture.as_surface(), filter);
            }
        }

        self.frame_count += 1;

        Ok(())
    }
}

struct IsfUniforms<'a, U: Uniforms> {
    frame_index: u32,
    time_delta: f32,
    time: f32,
    pass_index: i32,
    passes: &'a Vec<PassTexture>,
    inner: &'a U,
}

impl <U: Uniforms> Uniforms for IsfUniforms<'_, U> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        f("FRAMEINDEX", UniformValue::SignedInt(self.frame_index as i32));
        f("TIMEDELTA", self.time_delta.as_uniform_value());
        f("TIME", self.time.as_uniform_value());
        f("PASSINDEX", self.pass_index.as_uniform_value());
        for PassTexture { pass, texture } in self.passes {
            if let Some(name) = pass.target.as_ref() {
                f(name, texture.as_uniform_value());
            }
        }
        self.inner.visit_values(f);
    }
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

#[derive(Error, Debug)]
pub enum IsfShaderLoadError {
    #[error("Load error {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Compile error {0}")]
    CompileError(#[from] GlProgramCreationError),

    #[error("Parse error {0}")]
    PassParseError(#[from] PassParseError)
}