use glam::Vec3;
use glium::{
    implement_vertex, index, uniform, Blend, Display, DrawParameters, Program, Smooth, Surface,
    VertexBuffer,
};

use super::modular_shader::ModularShader;

#[derive(Copy, Clone)]
pub struct InstanceAttr {
    pub instance_pos: [f32; 3],
    pub instance_rgba: [f32; 4],
    pub instance_scale: [f32; 2],
}
implement_vertex!(InstanceAttr, instance_pos, instance_rgba, instance_scale);

#[derive(Copy, Clone)]
struct VertexAttr {
    position: [f32; 3],
}
implement_vertex!(VertexAttr, position);

type Mat4 = [[f32; 4]; 4];

pub struct InstancesView {
    vert_buffer: VertexBuffer<VertexAttr>,
    inst_buffer: VertexBuffer<InstanceAttr>,
    program: Program,
    verts_per_inst: usize,
    projection_mat: Mat4,
    params: DrawParameters<'static>,
}

impl InstancesView {
    pub fn new<I: Iterator<Item = T> + ExactSizeIterator, T: Into<InstanceAttr>>(
        display: &Display,
        source: I,
        projection_mat: Mat4
    ) -> Self {
        let program = Program::from_source(
            display,
            include_str!("instance.vert"),
            include_str!("instance.frag"),
            None,
        )
        .unwrap();

        let params = DrawParameters {
            dithering: true,
            smooth: Some(Smooth::Fastest),
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        let num_instances = source.len();

        let (vert_buffer, inst_buffer) = gen_buffers(display, source);

        let verts_per_inst = vert_buffer.len() / num_instances;

        Self {
            params,
            verts_per_inst,
            vert_buffer,
            inst_buffer,
            projection_mat,
            program
        }
    }

    pub fn write_instances<I>(&mut self, iter: I)
    where
        I: Iterator<Item = InstanceAttr>,
    {
        let mut mapping = self.inst_buffer.map();

        let zipped = mapping.chunks_exact_mut(self.verts_per_inst).zip(iter);

        for (chunk, from) in zipped {
            for inst in chunk {
                *inst = from;
            }
        }
    }
}

impl ModularShader for InstancesView {
    fn draw_to<S: Surface>(&self, surface: &mut S) -> Result<(), glium::DrawError>
    where
        Self: Sized 
    {
        surface.draw(
            (&self.vert_buffer, &self.inst_buffer),
            &index::NoIndices(index::PrimitiveType::TrianglesList),
            &self.program,
            &uniform! {
                persp_matrix: self.projection_mat,
            },
            &self.params,
        )
    }
}

fn gen_buffers<I: Iterator<Item = T> + ExactSizeIterator, T: Into<InstanceAttr>>(
    display: &Display,
    source: I,
) -> (VertexBuffer<VertexAttr>, VertexBuffer<InstanceAttr>) {
    let tri = [[-0.5, 0., 0.], [0., 1., 0.], [0.5, 0., 0.]].map(|slice| Vec3::from_slice(&slice));

    let tri_opp = tri.map(|pos| pos * -1.0);

    let vertices: Vec<VertexAttr> = [tri, tri_opp]
        .iter()
        .flatten()
        .map(|pos| VertexAttr {
            position: pos.to_array(),
        })
        .collect();

    let vertices_per_instance = vertices.len();

    let vert_data = std::iter::repeat(vertices)
        .take(source.len())
        .flatten()
        .collect::<Vec<_>>();

    let instance_data = source
        .flat_map(|intoAttrs| std::iter::repeat(intoAttrs.into()).take(vertices_per_instance))
        .collect::<Vec<_>>();

    let instances_buffer = glium::vertex::VertexBuffer::dynamic(display, &instance_data).unwrap();
    let vertices_buffer = glium::vertex::VertexBuffer::new(display, &vert_data).unwrap();

    (vertices_buffer, instances_buffer)
}
