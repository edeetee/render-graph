use std::any::Any;
use std::fmt::Display;
use std::{fs::File, io::Write};
use std::path::Path;
use glium::{uniforms::{Uniforms, UniformValue, AsUniformValue}, ProgramCreationError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

pub fn write_to_json_file(path: &Path, data: &impl Serialize) -> anyhow::Result<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, data)?;

    Ok(())
}

// pub fn write_to_toml_file(path: &Path, data: &impl Serialize) -> anyhow::Result<()> {
//     let mut file = File::create(path)?;
//     let mut output = String::new();
//     data.serialize(toml::Serializer::pretty(&mut output))?;
//     file.write_all(output.as_bytes())?;

//     Ok(())
// }

pub fn read_from_json_file<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let file = File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

pub trait MappableTuple<T> {
    type Ret<R>;
    fn map<F: Fn(&T) -> R, R>(&self, f: F) -> Self::Ret<R>;
}

impl <T> MappableTuple<T> for (T,T) {
    type Ret<R> = (R,R);

    fn map<F: Fn(&T) -> R, R>(&self, f: F) -> Self::Ret<R> {
        (f(&self.0), f(&self.0))
    }
}

pub struct MultiUniforms<'a, T: Uniforms> {
    // name: &'a str,
    // val: UniformValue<'a>,
    pub uniforms: Vec<(&'a str, UniformValue<'a>)>,
    pub next: &'a T
}

impl <'a, T: Uniforms> MultiUniforms<'a,T> {
    pub fn single<V: AsUniformValue>(name: &'a str, val: &'a V, other: &'a T) -> Self {
        Self {
            uniforms: vec![(name, val.as_uniform_value())],
            next: other
        }
    }
}

impl<'b,T: Uniforms> Uniforms for MultiUniforms<'b,T>{
    fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(&'a self, mut output: F) {
        // output("res", self.res.as_uniform_value());
        for (name, val) in &self.uniforms {
            output(name, *val);
        }
        self.next.visit_values(output);
    }
}

pub trait ToGlCreationError {
    fn to_gl_creation_error(self, shader_source: String) -> GlProgramCreationError;
}

impl ToGlCreationError for ProgramCreationError {
    fn to_gl_creation_error(self, shader_source: String) -> GlProgramCreationError {
        GlProgramCreationError {
            shader_source,
            inner: self
        }
    }
}

#[derive(Error, Debug)]
pub struct GlProgramCreationError{
    shader_source: String,
    pub inner: ProgramCreationError
}

impl std::fmt::Display for GlProgramCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            glium::ProgramCreationError::CompilationError(source, shader_type) => {
                write!(f, "CompilationError for {shader_type:?} (\n{source})")
            }
            glium::ProgramCreationError::LinkingError(source) => {
                write!(f, "LinkingError (\n{source})")
            },
            _ => self.inner.fmt(f),
        }
    }
}