use std::{time::{Instant, Duration}};
use glam::Vec3;
use glium::{glutin::{self, window::Fullscreen, event}, Surface, implement_vertex, uniform, VertexBuffer, Display, Texture2d, uniforms::{self, MagnifySamplerFilter}, Smooth, Program, index, texture::{UncompressedFloatFormat, UncompressedIntFormat, self}, backend::{Context, Facade}, Blend};
use rand::Rng;
use stars::Stars;
use crate::model::UpdateInfo;

use super::model;

pub struct Options{
    pub num_stars: usize
}

pub fn main(options: Options) {
    let mut model = model::Model::new(options.num_stars);

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();

    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    let monitor_handle = display.gl_window().window().available_monitors().next().unwrap();
    let fs = Fullscreen::Borderless(Some(monitor_handle));
    display.gl_window().window().set_fullscreen(Some(fs));

    // print_formats(&display.get_context());

    let program = glium::Program::from_source(
        &display, 
        include_str!("instance.vert"), 
        include_str!("instance.frag"), 
        None
    ).unwrap();

    let (vertices_buffer, mut instances_buffer) = gen_data(&display, &model.stars);
    let vertices_per_instance = vertices_buffer.len()/model.stars.iter().len();

    let fullscreen_vertices = FULLSCREEN_TRI
        .map(|arr| VertexAttr {position: arr} );
    let fullscreen_buffer = VertexBuffer::new(&display, &fullscreen_vertices).unwrap();

    let fullscreen_feedback_program = Program::from_source(
        &display,
        include_str!("feedback.vert"),
        include_str!("feedback.frag"),
        None
    ).unwrap();

    let update_period = Duration::from_millis(20);
    let mut last_update = Instant::now();
    let mut frames_since_update: u32 = 0;

    let mut res: [f32; 2] = get_res(&display).map(|a| a as f32);
    let mut feedback_displace: [f32; 2] = [0.0, 0.0];

    let feedback_texture = gen_texture(&display);
    let draw_texture = gen_texture(&display);

    event_loop.run(move |ev, _, control_flow| {
        
        let now = Instant::now();
        let elapsed_time = now-last_update;

        //update instances
        if update_period < elapsed_time{
            last_update = now;

            // feedback_displace = [rand::thread_rng().gen_range(-1f32..1f32), rand::thread_rng().gen_range(-1f32..1f32)];

            model.update(UpdateInfo{
                time_since_previous: elapsed_time,
                frames_since_previous: frames_since_update,
            });

            let mut mapping = instances_buffer.map();
            let mapping_iter = mapping.chunks_exact_mut(vertices_per_instance);
            
            for (buf, star) in mapping_iter.zip(model.stars.iter()) {
                for attr in buf {
                    attr.instance_pos = star.pos.to_array();
                }
            }

            frames_since_update = 0;
        }

        frames_since_update += 1;

        // drawing a frame
        let params = glium::DrawParameters {
            // depth: glium::Depth {
            //     test: glium::DepthTest::IfLess,
            //     write: true,
            //     .. Default::default()
            // },
            dithering: true,
            smooth: Some(Smooth::Fastest),
            blend: Blend::alpha_blending(),
            .. Default::default()
        };

        //todo: multidraw?
        //https://www.khronos.org/opengl/wiki/Vertex_Rendering#Indirect_rendering
        //copy vertices per instance??
        
        let mut screen_surface = display.draw();
        screen_surface.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        let mut draw_surface = draw_texture.as_surface();
        screen_surface.clear_color(0.0, 0.0, 0.0, 0.0);

        //draw feedback
        let feedback_sampler = feedback_texture.sampled()
            .wrap_function(uniforms::SamplerWrapFunction::BorderClamp);

            draw_surface.draw(
            &fullscreen_buffer,
            &index::NoIndices(index::PrimitiveType::TrianglesList),
            &fullscreen_feedback_program,
            &uniform! {
                feedback_texture: feedback_sampler,
                size: res,
                displace: feedback_displace
            },
            &params
        ).unwrap();

        //draw objects
        draw_surface.draw(
            (&vertices_buffer, &instances_buffer),
            &index::NoIndices(index::PrimitiveType::TrianglesList), 
            &program, 
            &uniform! { 
                persp_matrix: model.mat.to_cols_array_2d(),
            },
            &params
        ).unwrap();

        //copy to feedback
        feedback_texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
        draw_surface.fill(&feedback_texture.as_surface(), glium::uniforms::MagnifySamplerFilter::Linear);

        draw_surface.fill(&screen_surface, glium::uniforms::MagnifySamplerFilter::Linear);

        screen_surface.finish().unwrap();

        let next_frame_time = std::time::Instant::now() +
        std::time::Duration::from_millis(1000/60);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match ev {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                event::WindowEvent::Resized(size) => {
                    res = [size.width as f32, size.height as f32];
                }
                event::WindowEvent::KeyboardInput { input, .. } => {
                    if input.virtual_keycode == Some(event::VirtualKeyCode::Escape) {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                }
                _ => {},
            },
            _ => (),
        }
    });
}

fn get_res(display: &Display) -> [u32; 2] {
    let (w, h) = display.get_framebuffer_dimensions();
    return [w, h];
}

fn gen_texture(display: &Display) -> Texture2d {
    let (width, height) = display.get_framebuffer_dimensions();

    let texture = Texture2d::empty_with_format(
        display, 
        glium::texture::UncompressedFloatFormat::U16U16U16U16, 
        glium::texture::MipmapsOption::NoMipmap, 
        width, height
    ).unwrap();

    texture.as_surface().clear_color(0.0, 0.0, 0.0, 1.0);

    texture
}

fn gen_data(display: &Display, stars: &Stars) -> (VertexBuffer<VertexAttr>, VertexBuffer<InstanceAttr>) {
    const VERT_SCALE: f32 = 0.01;

    let tri = [
        [-0.5, -0., 0.],
        [ 0.,  1., 0.],
        [ 0.5, -0., 0.],
    ].map(|slice| Vec3::from_slice(&slice)*VERT_SCALE);
 
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
                instance_color: star.color,
                instance_radius: star.radius
            }
        ).take(vertices_per_instance)
    }).collect::<Vec<_>>();

    let instances_buffer = glium::vertex::VertexBuffer::dynamic(display, &instance_data).unwrap();
    let vertices_buffer = glium::vertex::VertexBuffer::new(display, &vert_data).unwrap();

    (vertices_buffer, instances_buffer)
}

fn print_formats(context: &Context){
    let all_formats = texture::UncompressedFloatFormat::get_formats_list();
    let valid_formats = all_formats.iter()
        .filter(|format| format.is_color_renderable(context) && format.is_supported(context));

    if valid_formats.clone().count() != 0 {
        println!("Valid formats:");
        for format in valid_formats {
            println!("{format:?}")
        }
    } else {
        println!("No valid formats!");
    }
}

#[derive(Copy, Clone)]
struct InstanceAttr {
    instance_pos: [f32; 3],
    instance_color: [f32; 3],
    instance_radius: f32
}
implement_vertex!(InstanceAttr, instance_pos, instance_color, instance_radius);

#[derive(Copy, Clone)]
struct VertexAttr {
    position: [f32; 3]
}
implement_vertex!(VertexAttr, position);

const FULLSCREEN_TRI: [[f32; 3]; 3] = [
    [-1.0, -1.0, 0.0],
    [3.0, -1.0, 0.0],
    [-1.0, 3.0, 0.0]
];