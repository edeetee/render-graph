use std::{time::Instant};


use glium::{VertexBuffer, implement_vertex, index::{self}, backend::Facade, Program, DrawParameters, Smooth, Blend, DrawError, Surface, uniforms::{Uniforms, AsUniformValue}, ProgramCreationError, IndexBuffer, Depth, BackfaceCullingMode};
use tri_mesh::{MeshBuilder, prelude::Mesh};

use crate::{util::MultiUniforms, textures::DEFAULT_RES};

pub fn new_vertex_buffer(facade: &impl Facade, verts: &[VertexAttr]) -> VertexBuffer<VertexAttr> {
    VertexBuffer::immutable(facade, verts).unwrap()
}

pub fn new_index_buffer(facade: &impl Facade, indices: &[u32]) -> IndexBuffer<u32> {
    IndexBuffer::immutable(facade, index::PrimitiveType::TrianglesList, indices).unwrap()
}

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
            polygon_offset: glium::draw_parameters::PolygonOffset{
                factor: 2.0,
                fill: true,
                ..Default::default()
            },
            depth: Depth {
                test: glium::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            .. Default::default()
        };

        Self::new_with_params(facade, params)
    }

    pub fn new_with_params(facade: &impl Facade, params: DrawParameters<'static>) -> Result<Self, ProgramCreationError> {
        let mesh = MeshBuilder::new().cube().build().unwrap();

        let vertices: Vec<_> = vertices_from_mesh(&mesh);

        let indices = mesh.indices_buffer();

        // let default_obj = Object::new(facade, &vertices, &indices);
    
        let program = Program::from_source(
            facade,
            &include_str!("obj.vert"),
            &include_str!("obj.frag"),
            None
        )?;

        let proj_matrix = glam::Mat4::perspective_rh(
            std::f32::consts::FRAC_2_PI, 
            DEFAULT_RES.0 as f32/DEFAULT_RES.1 as f32, 
            0.01, 
            100.0
        ).to_cols_array_2d();

        Ok(Self{
            params,
            start: Instant::now(),
            vert_buffer: new_vertex_buffer(facade, &vertices),
            index_buffer: new_index_buffer(facade, &indices),
            program,
            proj_matrix
        })
    }

    pub fn set_tri_data(&mut self, facade: &impl Facade, verts: &[VertexAttr], indices: &[u32]) {
        
        self.vert_buffer = new_vertex_buffer(facade, verts);
        self.index_buffer = new_index_buffer(facade, indices);
    }

    /// Draws the object to the surface
    /// !!! MUST contain a depth buffer
    pub fn draw(&self, surface: &mut impl Surface, uniforms: &impl Uniforms) -> Result<(), DrawError>{
        // let dim = surface.get_dimensions();
        let time = Instant::now() - self.start;

        let pre_matrix = glam::Mat4::from_rotation_y(time.as_secs_f32()*0.1);

        let view_matrix = glam::Mat4::look_at_rh(
            glam::Vec3::new(0.0, -2.0, 5.0),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );

        let _matrix = (view_matrix*pre_matrix).to_cols_array_2d();

        let combo_uniforms = MultiUniforms {
            uniforms: vec![
                // ("view_matrix", matrix.as_uniform_value()),
                ("proj_matrix", self.proj_matrix.as_uniform_value())
            ],
            next: uniforms
        };

        surface.draw(
            &self.vert_buffer,
            &self.index_buffer,
            &self.program,
            &combo_uniforms,
            &self.params
        )?;

        Ok(())
    }
}

// pub fn buffers_from_data<V: glium::Vertex,I: glium::index::Index>(facade: &impl Facade, verts: &[V], indices: &[I]) -> (glium::VertexBuffer<V>, glium::IndexBuffer<I>) {
//     let vert_buffer = VertexBuffer::immutable(facade, verts).unwrap();

//     let index_buffer = IndexBuffer::immutable(facade, index::PrimitiveType::TrianglesList, indices).unwrap();

//     (vert_buffer, index_buffer)
// }

pub fn vertices_from_mesh(mesh: &Mesh) -> Vec<VertexAttr> {
    mesh.vertex_iter()
        .map(|id| mesh.vertex_position(id).map(|n| n as f32))
        .map(|v| VertexAttr { position: v.into() })
        .collect()
}

#[derive(Copy, Clone)]
pub struct VertexAttr {
    pub position: [f32; 3]
}

implement_vertex!(VertexAttr, position);