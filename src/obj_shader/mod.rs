use std::{f32::consts::FRAC_PI_2, time::Instant};

use glium::{VertexBuffer, implement_vertex, index::{self}, backend::Facade, Program, DrawParameters, Smooth, Blend, DrawError, Surface, uniforms::{Uniforms, AsUniformValue, EmptyUniforms, UniformValue}, ProgramCreationError, IndexBuffer, uniform, Depth, BackfaceCullingMode};
use tri_mesh::{MeshBuilder, prelude::Mesh};
pub struct ObjRenderer{
    program: Program,
    vert_buffer: VertexBuffer<VertexAttr>,
    index_buffer: IndexBuffer<u32>,
    params: DrawParameters<'static>,
    start: Instant,
    proj_matrix: [[f32; 4]; 4],
}

impl ObjRenderer {
    pub fn new(facade: &impl Facade) -> Result<Self, ProgramCreationError> {
        let params = DrawParameters {
            dithering: true,
            smooth: Some(Smooth::Fastest),
            blend: Blend::alpha_blending(),
            backface_culling: BackfaceCullingMode::CullClockwise,
            // depth: Depth {
            //     test: glium::DepthTest::IfLessOrEqual,
            //     write: true,
            //     ..Default::default()
            // },
            .. Default::default()
        };

        Self::new_with_params(facade, params)
    }

    pub fn new_with_params(facade: &impl Facade, params: DrawParameters<'static>) -> Result<Self, ProgramCreationError> {
        let mesh = MeshBuilder::new().cube().build().unwrap();

        let (vert_buffer, index_buffer) = buffers_from_mesh(facade, mesh);
    
        let program = Program::from_source(
            facade,
            &include_str!("obj.vert"),
            &include_str!("obj.frag"),
            None
        )?;

        let proj_matrix = glam::Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_4, 
            2.0, 
            0.0001, 
            100.0
        ).to_cols_array_2d();

        Ok(Self{
            params,
            start: Instant::now(),
            vert_buffer,
            index_buffer,
            program,
            proj_matrix
        })
    }

    pub fn set_mesh(&mut self, facade: &impl Facade, mesh: Mesh) {
        // self.vert_buffer.
        (self.vert_buffer, self.index_buffer) = buffers_from_mesh(facade, mesh);
    }

    pub fn draw(&self, surface: &mut impl Surface) -> Result<(), DrawError>{
        // let dim = surface.get_dimensions();
        let time = Instant::now() - self.start;

        let pre_matrix = glam::Mat4::from_rotation_y(time.as_secs_f32()*0.1);

        let view_matrix = glam::Mat4::look_at_rh(
            glam::Vec3::new(0.0, -2.0, 5.0),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );

        surface.draw(
            &self.vert_buffer,
            &self.index_buffer,
            &self.program,
            &uniform! {
                view_matrix: (view_matrix*pre_matrix).to_cols_array_2d(),
                proj_matrix: self.proj_matrix,
            },
            &self.params
        )
    }
}

pub fn buffers_from_mesh(facade: &impl Facade, mesh: Mesh) -> (glium::VertexBuffer<VertexAttr>, glium::IndexBuffer<u32>) {
    let vertices: Vec<_> = mesh.vertex_iter()
        .map(|id| mesh.vertex_position(id).map(|n| n as f32))
        .map(|tri_mesh::prelude::Vector3{x,y,z}| VertexAttr{position: [x,y,z]})
        .collect();

    let vert_buffer = VertexBuffer::immutable(facade, vertices.as_slice()).unwrap();

    let indices = mesh.indices_buffer();

    let index_buffer = IndexBuffer::immutable(facade, index::PrimitiveType::TrianglesList, indices.as_slice()).unwrap();

    (vert_buffer, index_buffer)
}

#[derive(Copy, Clone)]
pub struct VertexAttr {
    position: [f32; 3]
}
implement_vertex!(VertexAttr, position);