use glium::{backend::Facade, Surface, uniforms::Uniforms, program::UniformBlock};
use glsl::{syntax::{Expr, ShaderStage, Identifier}, parser::Parse};

use crate::fullscreen_shader::FullscreenFrag;

pub struct GlExpressionRenderer {
    frag: Option<FullscreenFrag>
}

impl GlExpressionRenderer {
    pub fn new(_facade: &impl Facade) -> Self {
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
                eprintln!("Compiling '{shader}': Error {err}\nwhen ");
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

///Always defaults to Vec4
#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum GlType {
    Float,
    Vec2,
    Vec3,
    Vec4
}

impl GlType {
    fn from_length(len: usize) -> GlType {
        match len {
            1 => Self::Float,
            2 => Self::Vec2,
            3 => Self::Vec3,
            _ => Self::Vec4
        }
    }

    // fn from_iden(iden: &str) -> GlType {
    //     match iden {
    //         "pixel" => Self::Vec4,

    //     }
    // }
}

fn parse_gl_type(expr: &Expr) -> GlType {
    match expr {
        Expr::FloatConst(_) => GlType::Float,
        Expr::Dot(_, Identifier(iden)) => GlType::from_length(iden.len()),
        Expr::FunCall(glsl::syntax::FunIdentifier::Identifier(Identifier(fun_ident)), _) => {
            match fun_ident.as_str() {
                "vec2" => GlType::Vec2,
                "vec3" => GlType::Vec3,
                _ => GlType::Vec4
            }
        }
        Expr::Binary(_, a, b) => parse_gl_type(a).max(parse_gl_type(b)),
        _ => GlType::Vec4
    }
}

fn build_shader_from_snippet(snippet: &str) -> String {
    let expression = Expr::parse(snippet);
    dbg!(&expression);

    let snippet = snippet.to_string();

    let wrapped_snippet = match &expression {
        Ok(expr) => {
            match parse_gl_type(expr) {
                GlType::Float => format!("vec4(vec3({snippet}),1)"),
                GlType::Vec2 => format!("vec4({snippet}, 0,1)"),
                GlType::Vec3 => format!("vec4({snippet},1"),
                GlType::Vec4 => snippet
            }
        },
        Err(err) => {
            eprintln!("{}", err);
            snippet
        }
    };

    let shader_str = format!("
    #version 140
    uniform sampler2D pixels;
    
    void main() {{
        vec4 pixel = texture2D(pixels, gl_FragCoord.xy/textureSize(pixels, 0));
        gl_FragColor = {wrapped_snippet};
    }}
    ");

    // let stage = ShaderStage::parse(&shader_str);
    // dbg!(stage);

    shader_str
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