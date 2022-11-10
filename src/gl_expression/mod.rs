use glium::{backend::Facade, Surface, uniforms::Uniforms, program::UniformBlock, DrawError};
use glsl::{syntax::{Expr, Identifier}, parser::Parse};
use naga::{front::glsl::Options, ShaderStage};

use crate::{fullscreen_shader::FullscreenFrag, util::GlProgramCreationError};

pub struct GlExpressionRenderer {
    frag: Option<FullscreenFrag>
}

impl GlExpressionRenderer {
    pub fn new(_facade: &impl Facade) -> Self {
        Self {
            frag: None
        }
    }

    pub fn set_shader(&mut self, facade: &impl Facade, shader: &str) -> Result<Vec<(String, UniformBlock)>, GlProgramCreationError> {
        let full_source = build_shader_from_snippet(shader);

        let frag = FullscreenFrag::new(facade, &full_source)?;
        let uniform_data = frag.program.get_shader_storage_blocks().iter().map(|(n,v)| (n.clone(), v.clone())).collect();
        self.frag = Some(frag);

        Ok(uniform_data)
    }

    pub fn draw(&self, surface: &mut impl Surface, uniforms: &impl Uniforms) -> Result<(), DrawError> {
        if let Some(frag) = &self.frag {
            frag.draw(surface, uniforms)?
        }
        Ok(())
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
}

fn parse_gl_type(expr: &Expr) -> GlType {
    match expr {
        Expr::FloatConst(_) => GlType::Float,
        Expr::Dot(_, Identifier(iden)) => GlType::from_length(iden.len()),
        Expr::FunCall(glsl::syntax::FunIdentifier::Identifier(Identifier(fun_ident)), _) => {
            match fun_ident.as_str() {
                "vec2" => GlType::Vec2,
                "vec3" => GlType::Vec3,
                // "length" => GlType::Float,
                _ => GlType::Vec4
            }
        }
        Expr::Binary(_, a, b) => parse_gl_type(a).max(parse_gl_type(b)),
        _ => GlType::Vec4
    }
}

fn build_shader_from_snippet(snippet: &str) -> String {
    let mut parser = naga::front::glsl::Parser::default();

    let expression = Expr::parse(snippet);
    // dbg!(&expression);

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

    out vec4 out_color;
    
    void main() {{
        vec4 pixel = texture(pixels, gl_FragCoord.xy/textureSize(pixels, 0));
        out_color = {wrapped_snippet};
    }}
    ");

    let naga_result = parser.parse(
        &ShaderStage::Fragment.into(),
        &shader_str
    );

    // dbg!(&naga_result);

    // let stage = ShaderStage::parse(&shader_str);
    // dbg!(stage);

    shader_str
}

pub struct GlExpressionUpdater {
    pub frag_source: Option<String>
}

impl GlExpressionUpdater {
    pub fn update(&mut self, facade: &impl Facade, renderer: &mut GlExpressionRenderer, new_frag: String) -> Result<(), GlProgramCreationError> {

        let should_update_frag = match &self.frag_source {
            Some(shader) => shader != &new_frag,
            None => true
        };
        
        if should_update_frag {
            let result = renderer.set_shader(facade, &new_frag)?;
            self.frag_source = Some(new_frag);
        }

        Ok(())
    }
}