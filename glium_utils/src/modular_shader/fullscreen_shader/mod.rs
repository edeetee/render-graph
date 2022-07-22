use glium::{VertexBuffer, implement_vertex, index::{self, NoIndices}, backend::Facade, Program, DrawParameters, Display, Smooth, Blend, DrawError, Surface, uniforms::Uniforms};

pub struct FullscreenFrag{
    verts: VertexBuffer<VertexAttr>,
    program: Program,
    params: DrawParameters<'static>
}

impl FullscreenFrag {
    pub fn new(display: &Display, frag: &'static str) -> Self {
        let params = DrawParameters {
            dithering: true,
            smooth: Some(Smooth::Fastest),
            blend: Blend::alpha_blending(),
            .. Default::default()
        };

        Self::new_with_params(display, frag, params)
    }

    pub fn new_with_params(display: &Display, frag: &'static str, params: DrawParameters<'static>) -> Self {
        let vert_buffer = new_fullscreen_buffer(display).unwrap();
    
        let program = Program::from_source(
            display,
            FULLSCREEN_VERT_SHADER,
            frag,
            None
        ).unwrap();

        Self{
            params,
            verts: vert_buffer,
            program
        }
    }

    pub fn draw<S: Surface, U: Uniforms>(&self, surface: &mut S, uniforms: &U) -> Result<(), DrawError>{
        surface.draw(
            &self.verts,
            &FULLSCREEN_INDICES,
            &self.program,
            uniforms,
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