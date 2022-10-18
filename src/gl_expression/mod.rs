use glium::{backend::Facade, Surface, uniforms::Uniforms, program::UniformBlock};

use crate::fullscreen_shader::FullscreenFrag;

pub struct GlExpressionRenderer {
    frag: Option<FullscreenFrag>
}

impl GlExpressionRenderer {
    pub fn new(facade: &impl Facade) -> Self {
        Self {
            frag: None
        }
    }

    pub fn set_shader(&mut self, facade: &impl Facade, shader: &str) -> Option<Vec<(String, UniformBlock)>> {
        let full_source = build_shader_from_snippet(shader);

        match FullscreenFrag::new(facade, &full_source) {
            Ok(frag) => {
                let uniform_data = frag.program.get_shader_storage_blocks().iter().map(|(n,v)| (n.clone(), v.clone())).collect();
                self.frag = Some(frag);
                Some(uniform_data)
            },
            Err(err) => {
                eprintln!("Error {err} compiling {shader}\n");
                None
            }
        }
    }

    pub fn draw(&self, surface: &mut impl Surface, uniforms: &impl Uniforms) {
        if let Some(frag) = &self.frag {
            match frag.draw(surface, uniforms) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("Failed to run shader:\n{err}")
                }
            }
        }
    }
}

fn build_shader_from_snippet(snippet: &str) -> String {
    format!("
    #version 140
    uniform sampler2D pixels;
    
    void main() {{
        vec4 pixel = texture2D(pixels, gl_FragCoord.xy/textureSize(pixels, 0));
        gl_FragColor = {snippet};
    }}
    ")
}

pub struct GlExpressionUpdater {
    pub frag_source: Option<String>
}

impl GlExpressionUpdater {
    pub fn update(&mut self, facade: &impl Facade, renderer: &mut GlExpressionRenderer, new_frag: String) -> Option<Vec<(String, UniformBlock)>> {
        if match &self.frag_source {
            Some(shader) => shader != &new_frag,
            None => true
        } {
            let result = renderer.set_shader(facade, &new_frag);
            self.frag_source = Some(new_frag);

            result
        } else {
            None
        }
    }
}