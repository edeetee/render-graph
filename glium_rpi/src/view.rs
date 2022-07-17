// use glam::Vec3;
// use glium::{Display, vertex, VertexBuffer, implement_vertex, Program, Texture2d, Surface, index, uniform, uniforms, DrawParameters, Smooth, Blend};
// use stars::Stars;
// use crate::util::*;

// #[derive(Copy, Clone)]
// struct InstanceAttr {
//     instance_pos: [f32; 3],
//     instance_rgba: [f32; 4],
//     instance_radius: f32
// }
// implement_vertex!(InstanceAttr, instance_pos, instance_rgba, instance_radius);

// #[derive(Copy, Clone)]
// struct VertexAttr {
//     position: [f32; 3]
// }
// implement_vertex!(VertexAttr, position);

// const FULLSCREEN_TRI: [[f32; 3]; 3] = [
//     [-1.0, -1.0, 0.0],
//     [3.0, -1.0, 0.0],
//     [-1.0, 3.0, 0.0]
// ];

// pub struct View<'a> {
//     pub vert_buffer: VertexBuffer<VertexAttr>,
//     pub inst_buffer: VertexBuffer<InstanceAttr>,
//     fullscreen_buffer: VertexBuffer<VertexAttr>,
//     pub draw_texture: Texture2d,
//     pub feedback_texture: Texture2d,
//     pub draw_program: Program,
//     pub feedback_program: Program,
//     pub vert_per_inst: usize,
//     pub res: [f32; 2],
//     pub display: Display,
//     params: DrawParameters<'a>
// }

// impl View<'_>{
//     pub fn new(display: Display, stars: &Stars) -> Self {
//         let draw_program = glium::Program::from_source(
//             &display, 
//             include_str!("instance.vert"), 
//             include_str!("instance.frag"), 
//             None
//         ).unwrap();
    
//         let (vert_buffer, inst_buffer) = gen_data(&display, stars);
    
//         let fullscreen_verts = FULLSCREEN_TRI
//             .map(|arr| VertexAttr {position: arr} );
//         let fullscreen_buffer = VertexBuffer::new(&display, &fullscreen_verts).unwrap();
    
//         let feedback_program = Program::from_source(
//             &display,
//             include_str!("feedback.vert"),
//             include_str!("feedback.frag"),
//             None
//         ).unwrap();

//         let feedback_texture = gen_texture(&display);
//         let draw_texture = gen_texture(&display);
        
//         let mut res: [f32; 2] = get_res(&display).map(|a| a as f32);

//         let params = glium::DrawParameters {
//             dithering: true,
//             smooth: Some(Smooth::Fastest),
//             blend: Blend::alpha_blending(),
//             .. Default::default()
//         };

//         Self{
//             display,
//             vert_buffer,
//             inst_buffer,
//             feedback_texture,
//             fullscreen_buffer,
//             draw_texture,
//             feedback_program,
//             draw_program,
//             vert_per_inst: vert_buffer.len()/stars.iter().len(),
//             res,
//             params
//         }
//     }
// }