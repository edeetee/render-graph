

use egui::{Color32};
use egui_glium::EguiGlium;
use glium::glutin::{self, event::{Event, WindowEvent}, event_loop::ControlFlow, platform::{run_return::EventLoopExtRunReturn}, window::Fullscreen};
use crate::{common::persistent_state::{PersistentState, WindowState}};



use crate::editor::graph_ui::GraphUi;


// use super::{};

// const DEFAULT_FULLSCREEN_MODE: Option<Fullscreen> = Some(Fullscreen::Borderless(None));

// reference:
// https://github.com/emilk/egui/blob/master/egui_glium/examples/native_texture.rs

// const default_save_location = PathB

pub fn main() {
    // println!("CARGO PATH{}", env!("CARGO_PKG_NAME"));

    let state = PersistentState::load_from_default_path();

    let mut event_loop = glutin::event_loop::EventLoop::new();

    let display = create_display(&event_loop, &state.window);
    
    println!("GL Vendor: {}", display.get_opengl_vendor_string());
    println!("GL Version: {}", display.get_opengl_version_string());

    let mut egui_glium = EguiGlium::new(&display, &event_loop);

    // Ui::visual
    // egui_glium.egui_ctx.set_visuals(visuals)
    let mut visuals = egui_glium.egui_ctx.style().visuals.clone();
    visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    egui_glium.egui_ctx.set_visuals(visuals);

    let mut graph_ui = GraphUi::new_from_persistent(state, &display, &mut egui_glium);

    use signal_hook::consts::*;

    let mut exit_signals = signal_hook::iterator::Signals::new([
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ]).unwrap();

    event_loop.run_return(|ev, _, control_flow| {

        let exit = |control_flow: &mut ControlFlow| {
            *control_flow = glutin::event_loop::ControlFlow::Exit;
        };

        match ev {
            Event::RedrawRequested(_) => {
                // egui_glium.egui_winit.take_egui_input(window)
                graph_ui.process_frame(&display, &mut egui_glium);
            },
            Event::MainEventsCleared => {
                graph_ui.update(&display);

                if exit_signals.pending().count() != 0 {
                    exit(control_flow);
                }
            }
            Event::RedrawEventsCleared => {
                display.gl_window().window().request_redraw();
            }
            Event::WindowEvent { event: window_ev, .. } => {
                let egui_consumed_event = egui_glium.on_event(&window_ev);

                if !egui_consumed_event {
                    if matches!(window_ev, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                        exit(control_flow);
                    }
                }
            },
            _ => {}
        }
    });
    
    println!("EXITING");

    let res = logical_framebuffer_size(&display, &egui_glium.egui_ctx);
    let fullscreen = display.gl_window().window().fullscreen().is_some();

    let persistent_state = graph_ui.to_persistent(Some(WindowState{res, fullscreen}));
    let path = PersistentState::default_path();

    match persistent_state.write_to_default_path() {
        Ok(_) => {
            println!("Saved graph state to {path:?}")
        },
        Err(err) => {
            eprintln!("FAILED to save graph state to {path:?}\nERR({err:?})");
        }
    }
}

pub fn logical_framebuffer_size(glium: &glium::backend::Context, egui: &egui::Context) -> (u32, u32){
    let res = glium.get_framebuffer_dimensions();

    (
        (res.0 as f32/egui.pixels_per_point()) as u32, 
        (res.1 as f32/egui.pixels_per_point()) as u32
    )
}

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>, window_state: &Option<WindowState>) -> glium::Display {
    
    let _size = if let Some(WindowState { res, .. }) = window_state {
        glutin::dpi::LogicalSize {
            width: res.0 as f64,
            height: res.1 as f64,
        }
    } else {
        glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        }
    };
    
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("render-graph @optiphonic");

    let window_builder = if let Some(window_state) = window_state {
        window_builder.with_inner_size(glutin::dpi::LogicalSize {
            width: window_state.res.0 as f64,
            height: window_state.res.1 as f64,
        }).with_fullscreen(if window_state.fullscreen {Some(Fullscreen::Borderless(None))} else {None})
    } else {
        window_builder
    };
    
    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_hardware_acceleration(Some(true))
        .with_stencil_buffer(0)
        // .window
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}