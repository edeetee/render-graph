use std::{time::Instant, iter::{successors, from_fn}};

use glium::{Surface, VertexBuffer, Display, index::{PrimitiveType, NoIndices}, IndexBuffer};

#[macro_use]
extern crate glium;

use glium::glutin;
use stars::Stars;
mod model;

pub struct Options{
    pub num_stars: usize
}

pub fn main(options: Options) {

    let mut model = model::Model::new(options.num_stars);
    // let stars = model.stars;

    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
        // .with_gl(GlRequest::Specific(Api::OpenGlEs, (2, 1)));

    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // let (vertices, indices) = triangle_buffers(&display);

    let frag_shader = include_str!("instance.frag");
    let vert_shader = include_str!("instance.vert");

    let instance_data = gen_instance_attrs(&model.stars);    

    let vert_data = std::iter::repeat(VERTICES)
        .take(model.stars.iter().count())
        .flatten()
        .collect::<Vec<_>>();

    let program = glium::Program::from_source(
        &display, 
        vert_shader, 
        frag_shader, 
        None
    ).unwrap();

    let mut instances_buffer = glium::vertex::VertexBuffer::dynamic(&display, &instance_data).unwrap();
    let vertices_buffer = glium::vertex::VertexBuffer::new(&display, &vert_data).unwrap();

    let mut last_frame = Instant::now();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    event_loop.run(move |ev, _, control_flow| {
        
        let now = Instant::now();
        let elapsed_time = now-last_frame;
        last_frame = now;

        //update instances
        {
            model.update(elapsed_time.as_secs_f32());

            let mut mapping = instances_buffer.map();
            let mapping_iter = mapping.chunks_exact_mut(VERTICES.len());
            
            for (src, dest) in model.stars.iter().zip(mapping_iter) {
                for attr in dest {
                    attr.world_position = src.pos.to_array();
                }
            }
        }

        // drawing a frame
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        //todo: multidraw?
        //https://www.khronos.org/opengl/wiki/Vertex_Rendering#Indirect_rendering
        //copy vertices per instance??

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(
            (&vertices_buffer, &instances_buffer),
            &indices, 
            &program, 
            &uniform! { persp_matrix: model.mat.to_cols_array_2d() },
            &params
        ).unwrap();

        target.finish().unwrap();

        let next_frame_time = std::time::Instant::now() +
        std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            _ => (),
        }

        // action
    });
}

fn gen_instance_attrs(stars: &Stars) -> Vec<InstanceAttr> {
    stars.iter().map(|star| star.pos.to_array() ).flat_map(|data| {
        std::iter::repeat(
            InstanceAttr {
                world_position: data,
            }
        ).take(VERTICES.len())
    }).collect::<Vec<_>>()
}

#[derive(Copy, Clone)]
struct InstanceAttr {
    world_position: [f32; 3],
}
implement_vertex!(InstanceAttr, world_position);

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

const VERT_SCALE: f32 = 0.01;

const VERTICES: [Vertex; 3] = [
    Vertex { position: [-VERT_SCALE, -VERT_SCALE, 0.], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.0,  VERT_SCALE, 0.], color: [0.0, 0.0, 1.0] },
    Vertex { position: [ VERT_SCALE, -VERT_SCALE, 0.], color: [1.0, 0.0, 0.0] },
];