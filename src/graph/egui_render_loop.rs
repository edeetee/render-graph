use egui_glium::EguiGlium;
use glium::glutin::{self, event::{Event, WindowEvent}};

use super::ShaderGraphProcessor;

// const DEFAULT_FULLSCREEN_MODE: Option<Fullscreen> = Some(Fullscreen::Borderless(None));

// reference:
// https://github.com/emilk/egui/blob/master/egui_glium/examples/native_texture.rs

pub fn render_glium() {
    // let model = model::Model::new(options.num_stars, Some(options.model_options));
    // let model = ();

    let event_loop = glutin::event_loop::EventLoop::new();

    let display = create_display(&event_loop);

    println!("GL Vendor: {}", display.get_opengl_vendor_string());
    println!("GL Version: {}", display.get_opengl_version_string());

    let mut egui_glium = EguiGlium::new(&display);

    // egui_glium.egui_winit.egui_input().events
    
    // let (width, height) = display.get_framebuffer_dimensions();
    // let render_buffer = RenderBuffer::new(&display, DEFAULT_TEXTURE_FORMAT, width, height).unwrap();

    let mut shader_node_graph = ShaderGraphProcessor::new();

    event_loop.run(move |ev, _, control_flow| {
        
        shader_node_graph.update(&display);

        match ev {
            Event::RedrawRequested(_) => {
                // egui_glium.egui_winit.take_egui_input(window)
                shader_node_graph.draw(&display, &mut egui_glium);
            },
            Event::RedrawEventsCleared => {
                display.gl_window().window().request_redraw();
            }
            Event::WindowEvent { event: window_ev, .. } => {
                // egui_glium.egui_winit.
                let egui_consumed_event = egui_glium.on_event(&window_ev);

                if !egui_consumed_event {
                    if matches!(window_ev, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                }
            },
            _ => {}
        }
    });
}

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("egui_glium example");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_hardware_acceleration(Some(true))
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}