use std::time::Instant;

use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    Triangulate, Vertices,
};
use glium::{
    backend::Facade,
    implement_vertex,
    index::{self, IndexBufferAny},
    uniforms::{AsUniformValue, Uniforms},
    vertex::VertexBufferAny,
    BackfaceCullingMode, Blend, Depth, DrawError, DrawParameters, IndexBuffer, Program,
    ProgramCreationError, Smooth, Surface, VertexBuffer,
};

use crate::{textures::DEFAULT_RES, util::MultiUniforms};

pub fn new_vertex_buffer<T: glium::Vertex>(facade: &impl Facade, verts: &[T]) -> VertexBuffer<T> {
    VertexBuffer::immutable(facade, verts).unwrap()
}

pub fn new_index_buffer<T: glium::index::Index>(
    facade: &impl Facade,
    indices: &[T],
) -> IndexBuffer<T> {
    IndexBuffer::immutable(facade, index::PrimitiveType::TrianglesList, indices).unwrap()
}

pub struct ObjRenderer {
    program: Program,
    vert_buffer: VertexBufferAny,
    index_buffer: IndexBufferAny,
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
            polygon_offset: glium::draw_parameters::PolygonOffset {
                factor: 2.0,
                fill: true,
                ..Default::default()
            },
            depth: Depth {
                test: glium::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        Self::new_with_params(facade, params)
    }

    pub fn new_with_params(
        facade: &impl Facade,
        params: DrawParameters<'static>,
    ) -> Result<Self, ProgramCreationError> {
        // let mesh = MeshBuilder::new().cube().build().unwrap();

        let cube = genmesh::generators::Torus::new(1.0, 0.3, 30, 13);

        let vertices: Vec<_> = cube.shared_vertex_iter().map(PosNormVertex::from).collect();
        let indices: Vec<_> = cube
            .indexed_polygon_iter()
            .triangulate()
            .vertices()
            .map(|vertex| vertex as u32)
            .collect();

        let program = Program::from_source(
            facade,
            &include_str!("pos_and_norm.vert"),
            &include_str!("obj.frag"),
            None,
        )
        .unwrap();

        let proj_matrix = glam::Mat4::perspective_rh(
            std::f32::consts::FRAC_2_PI,
            DEFAULT_RES.0 as f32 / DEFAULT_RES.1 as f32,
            0.01,
            100.0,
        )
        .to_cols_array_2d();

        Ok(Self {
            params,
            start: Instant::now(),
            vert_buffer: new_vertex_buffer(facade, &vertices).into(),
            index_buffer: new_index_buffer(facade, &indices).into(),
            program,
            proj_matrix,
        })
    }

    pub fn update_data(&mut self, facade: &impl Facade, data: Data) {
        match data {
            Data::Pos(verts, indices) => {
                self.vert_buffer = new_vertex_buffer(facade, &verts).into();
                self.index_buffer = new_index_buffer(facade, &indices).into();
                self.program = Program::from_source(
                    facade,
                    &include_str!("pos_only.vert"),
                    &include_str!("obj.frag"),
                    None,
                )
                .unwrap();
            }
            Data::PosNorm(verts, indices) => {
                self.vert_buffer = new_vertex_buffer(facade, &verts).into();
                self.index_buffer = new_index_buffer(facade, &indices).into();
                self.program = Program::from_source(
                    facade,
                    &include_str!("pos_and_norm.vert"),
                    &include_str!("obj.frag"),
                    None,
                )
                .unwrap();
            }
        }
    }

    // pub fn update_positions_and_normals(&mut self, facade: &impl Facade, verts: &[PosNormVertex], indices: &[u32]) {
    //     self.vert_buffer = new_vertex_buffer(facade, verts).into();
    //     self.index_buffer = new_index_buffer(facade, indices);
    // }

    /// Draws the object to the surface
    /// !!! MUST contain a depth buffer
    pub fn draw(
        &self,
        surface: &mut impl Surface,
        uniforms: &impl Uniforms,
    ) -> Result<(), DrawError> {
        // let dim = surface.get_dimensions();
        let time = Instant::now() - self.start;

        let pre_matrix = glam::Mat4::from_rotation_y(time.as_secs_f32() * 0.1);

        let view_matrix = glam::Mat4::look_at_rh(
            glam::Vec3::new(0.0, -2.0, 5.0),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );

        let _matrix = (view_matrix * pre_matrix).to_cols_array_2d();

        let combo_uniforms = MultiUniforms {
            uniforms: vec![
                // ("view_matrix", matrix.as_uniform_value()),
                ("proj_matrix", self.proj_matrix.as_uniform_value()),
            ],
            next: uniforms,
        };

        surface.draw(
            &self.vert_buffer,
            &self.index_buffer,
            &self.program,
            &combo_uniforms,
            &self.params,
        )?;

        Ok(())
    }
}

pub enum Data {
    Pos(Vec<PosVertex>, Vec<u32>),
    PosNorm(Vec<PosNormVertex>, Vec<u32>),
}

#[derive(Copy, Clone)]
pub struct PosVertex {
    pub position: [f32; 3],
}

impl PosVertex {
    pub fn new(position: [f32; 3]) -> Self {
        Self { position }
    }
}

#[derive(Copy, Clone)]
pub struct PosNormVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl PosNormVertex {
    pub fn new((position, normal): ([f32; 3], [f32; 3])) -> Self {
        Self { position, normal }
    }
}

impl From<genmesh::Vertex> for PosNormVertex {
    fn from(value: genmesh::Vertex) -> Self {
        Self {
            position: [value.pos.x, value.pos.y, value.pos.z],
            normal: [value.normal.x, value.normal.y, value.normal.z],
        }
    }
}

impl From<genmesh::Vertex> for PosVertex {
    fn from(value: genmesh::Vertex) -> Self {
        Self {
            position: [value.pos.x, value.pos.y, value.pos.z],
        }
    }
}

implement_vertex!(PosVertex, position);
implement_vertex!(PosNormVertex, position, normal);
