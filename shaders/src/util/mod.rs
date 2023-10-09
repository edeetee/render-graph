use glium::{
    uniforms::{AsUniformValue, UniformValue, Uniforms},
    ProgramCreationError,
};

use thiserror::Error;

pub struct MultiUniforms<'a, T: Uniforms> {
    // name: &'a str,
    // val: UniformValue<'a>,
    pub uniforms: Vec<(&'a str, UniformValue<'a>)>,
    pub next: &'a T,
}

impl<'a, T: Uniforms> MultiUniforms<'a, T> {
    pub fn single<V: AsUniformValue>(name: &'a str, val: &'a V, other: &'a T) -> Self {
        Self {
            uniforms: vec![(name, val.as_uniform_value())],
            next: other,
        }
    }
}

impl<'b, T: Uniforms> Uniforms for MultiUniforms<'b, T> {
    fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(
        &'a self,
        mut output: F,
    ) {
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
            inner: self,
        }
    }
}

#[derive(Error, Debug)]
pub struct GlProgramCreationError {
    shader_source: String,
    pub inner: ProgramCreationError,
}

impl std::fmt::Display for GlProgramCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            glium::ProgramCreationError::CompilationError(source, shader_type) => {
                write!(f, "CompilationError for {shader_type:?} (\n{source})")
            }
            glium::ProgramCreationError::LinkingError(source) => {
                write!(f, "LinkingError (\n{source})")
            }
            _ => write!(f, "{} (\n{}\n)", self.inner, self.shader_source),
        }
    }
}
