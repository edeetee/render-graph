use glium::{VertexBuffer, implement_vertex, index::{self}, backend::Facade, Program, DrawParameters, Smooth, Blend, DrawError, Surface, uniforms::{Uniforms, AsUniformValue}, ProgramCreationError};
pub struct FullscreenFrag{
    verts: VertexBuffer<VertexAttr>,
    program: Program,
    params: DrawParameters<'static>
}

struct FullscreenUniforms<'a, U: Uniforms> {
    res: [f32; 2],
    inner: &'a U
}

impl<U: Uniforms> Uniforms for FullscreenUniforms<'_, U>{
    fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(&'a self, mut output: F) {
        output("res", self.res.as_uniform_value());
        self.inner.visit_values(output);
    }
}

impl FullscreenFrag {
    pub fn new(facade: &impl Facade, frag: &str) -> Result<Self, ProgramCreationError> {
        let params = DrawParameters {
            dithering: true,
            smooth: Some(Smooth::Fastest),
            blend: Blend::alpha_blending(),
            .. Default::default()
        };

        Self::new_with_params(facade, frag, params)
    }

    pub fn new_with_params(facade: &impl Facade, frag: &str, params: DrawParameters<'static>) -> Result<Self, ProgramCreationError> {
        let vert_buffer = new_fullscreen_buffer(facade).unwrap();
    
        let program = Program::from_source(
            facade,
            FULLSCREEN_VERT_SHADER,
            frag,
            None
        )?;

        Ok(Self{
            params,
            verts: vert_buffer,
            program
        })
    }

    pub fn draw(&self, surface: &mut impl Surface, uniforms: &impl Uniforms) -> Result<(), DrawError>{
        let dim = surface.get_dimensions();

        let extra_uniforms = FullscreenUniforms {
            res: [dim.0 as f32, dim.1 as f32],
            inner: uniforms
        };

        surface.draw(
            &self.verts,
            &FULLSCREEN_INDICES,
            &self.program,
            &extra_uniforms,
            &self.params
        )
    }
}

#[derive(Copy, Clone)]
pub struct VertexAttr {
    position: [f32; 3]
}
implement_vertex!(VertexAttr, position);

const fn v(x: f32, y: f32, z: f32) -> VertexAttr {
    VertexAttr{
        position: [x, y, z]
    }
}

const FULLSCREEN_VERTICES: [VertexAttr; 3] = [
    v(-1.0, -1.0, 0.0),
    v(3.0, -1.0, 0.0),
    v(-1.0, 3.0, 0.0)
];


pub const FULLSCREEN_INDICES: glium::index::NoIndices = index::NoIndices(index::PrimitiveType::TrianglesList);

pub fn new_fullscreen_buffer<F: ?Sized + Facade>(facade: &F)
    -> Result<VertexBuffer<VertexAttr>, glium::vertex::BufferCreationError> 
{
    VertexBuffer::immutable(facade, &FULLSCREEN_VERTICES)
}

pub const FULLSCREEN_VERT_SHADER: & 'static str = include_str!("fullscreen.vert");