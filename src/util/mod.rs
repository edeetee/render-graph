use glium::uniforms::{Uniforms, UniformValue, AsUniformValue};

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