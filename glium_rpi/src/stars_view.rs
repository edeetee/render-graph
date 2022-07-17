use std::slice::{ChunksExactMut, ChunksExact};

use glam::{Vec3, Mat4};
use glium::{Display, VertexBuffer, implement_vertex, Program, Surface, index, uniform, DrawParameters, Smooth, Blend, buffer::{Mapping, Content}};
use stars::Stars;

#[derive(Copy, Clone)]
pub struct InstanceAttr {
    pub instance_pos: [f32; 3],
    pub instance_rgba: [f32; 4],
    pub instance_radius: f32
}
implement_vertex!(InstanceAttr, instance_pos, instance_rgba, instance_radius);

#[derive(Copy, Clone)]
struct VertexAttr {
    position: [f32; 3]
}
implement_vertex!(VertexAttr, position);

pub struct StarsView {
    pub vert_buffer: VertexBuffer<VertexAttr>,
    pub inst_buffer: VertexBuffer<InstanceAttr>,
    pub program: Program,
    pub vert_per_inst: usize,
    params: DrawParameters<'static>
}

// struct OwnedChunksExactMut<'a, T>
//     where [T]: Content,
//         T: Copy
// {
//     mapping: Mapping<'a, [T]>,
//     chunked_iter: ChunksExactMut<'a, T>
// }

// impl<'a, T> OwnedChunksExactMut<'a, T>
// where [T]: Content,
//     T: Copy
// {
//     fn new(mut mapping: Mapping<'a, [T]>, chunk_size: usize) -> Self {
//         // let mut mapping = buffer.map();
//         let chunked_iter = mapping.chunks_exact_mut(chunk_size);

//         Self{
//             mapping,
//             chunked_iter
//         }
//     }
// }

// impl<'a, T: Copy> Iterator for OwnedChunksExactMut<'a, T>
// {
//     type Item = &'a mut [T];

//     fn next(&mut self) -> Option<Self::Item> {
//         self.chunked_iter.next()
//     }
// }


impl StarsView {
    pub fn new(display: &Display, stars: &Stars) -> Self {
        let program = Program::from_source(
            display, 
            include_str!("instance.vert"), 
            include_str!("instance.frag"), 
            None
        ).unwrap();
    
        let (vert_buffer, inst_buffer) = gen_data(display, stars);

        let params = glium::DrawParameters {
            dithering: true,
            smooth: Some(Smooth::Fastest),
            blend: Blend::alpha_blending(),
            .. Default::default()
        };

        let vert_per_inst = vert_buffer.len()/stars.iter().len();

        Self{
            vert_per_inst,
            vert_buffer,
            inst_buffer,
            program,
            params
        }
    }

    pub fn write_instances<I>(&mut self, iter: I)
        where I: Iterator<Item = InstanceAttr>
    {
        let mut mapping = self.inst_buffer.map();

        for (chunk, from) in mapping.chunks_exact_mut(self.vert_per_inst).zip(iter){
            for inst in chunk {
                inst.instance_pos = from.instance_pos;
                inst.instance_radius = from.instance_radius;
                inst.instance_rgba = from.instance_rgba
            }
        }
    }

    pub fn draw_to<S>(&self, surface: &mut S, mat: [[f32; 4]; 4]) -> Result<(), glium::DrawError>
        where S: Surface
    {
        surface.draw(
            (&self.vert_buffer, &self.inst_buffer),
            &index::NoIndices(index::PrimitiveType::TrianglesList), 
            &self.program, 
            &uniform! { 
                persp_matrix: mat,
            },
            &self.params
        )
    }
}


fn gen_data(display: &Display, stars: &Stars) -> (VertexBuffer<VertexAttr>, VertexBuffer<InstanceAttr>) {
    let tri = [
        [-0.5, -0., 0.],
        [ 0.,  1., 0.],
        [ 0.5, -0., 0.],
    ].map(|slice| Vec3::from_slice(&slice));
 
    let tri_opp = tri.map(|pos| pos*-1.0);

    let vertices: Vec<VertexAttr> = [tri, tri_opp].iter()
        .flatten()
        .map(|pos| VertexAttr { position: pos.to_array() })
        .collect();

    let vertices_per_instance = vertices.len();

    let vert_data = std::iter::repeat(vertices)
        .take(stars.iter().count())
        .flatten()
        .collect::<Vec<_>>();
        
    let instance_data = stars.iter().flat_map(|star| {
        std::iter::repeat(
            InstanceAttr {
                instance_pos: star.pos.to_array(),
                instance_rgba: star.rgba,
                instance_radius: star.radius
            }
        ).take(vertices_per_instance)
    }).collect::<Vec<_>>();

    let instances_buffer = glium::vertex::VertexBuffer::dynamic(display, &instance_data).unwrap();
    let vertices_buffer = glium::vertex::VertexBuffer::new(display, &vert_data).unwrap();

    (vertices_buffer, instances_buffer)
}