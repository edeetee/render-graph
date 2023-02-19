use std::{path::{Path}, env, fs::File};

use egui_glium::EguiGlium;
use glium::glutin::{self, event::{Event, WindowEvent}, event_loop::ControlFlow};
use serde::{Serialize, de::DeserializeOwned};

use crate::graph::{graph::{ShaderGraph, self}, ShaderGraphProcessor, def::{EditorState, ShaderNodeResponse}, renderer::Renderer};
use crate::util::{read_from_json_file, write_to_json_file};

// use super::{};

// const DEFAULT_FULLSCREEN_MODE: Option<Fullscreen> = Some(Fullscreen::Borderless(None));

// reference:
// https://github.com/emilk/egui/blob/master/egui_glium/examples/native_texture.rs

// const default_save_location = PathB

pub fn render_glium() {
    let default_save_path = env::current_exe().unwrap().parent().unwrap().join("render-graph-auto-save.json");

    let event_loop = glutin::event_loop::EventLoop::new();

    let display = create_display(&event_loop);
    
    println!("GL Vendor: {}", display.get_opengl_vendor_string());
    println!("GL Version: {}", display.get_opengl_version_string());

    let renderer = Renderer::new(&display);

    use signal_hook::consts::*;

    let mut exit_signals = signal_hook::iterator::Signals::new([
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ]).unwrap();

    event_loop.run(move |ev, _, control_flow| {

        let exit = |control_flow: &mut ControlFlow, graph_state: &EditorState| {
            // shader_node_graph.
            println!("EXITING");

            match write_to_json_file(&default_save_path, graph_state) {
                Ok(_) => {
                    println!("Saved graph state to {default_save_path:?}")
                },
                Err(err) => {
                    eprintln!("FAILED to save graph state to {default_save_path:?}\n{err:?}");
                }
            }
    
            *control_flow = glutin::event_loop::ControlFlow::Exit;
        };

        match ev {
            Event::RedrawRequested(_) => {
                // egui_glium.egui_winit.take_egui_input(window)
                shader_node_graph.draw(&display, &mut egui_glium);
            },
            Event::MainEventsCleared => {
                shader_node_graph.update(&display);

                if exit_signals.pending().count() != 0 {
                    exit(control_flow, &shader_node_graph.graph.editor);
                }
            }
            Event::RedrawEventsCleared => {
                display.gl_window().window().request_redraw();
            }
            // Event::WindowEvent { window_id, event }
            Event::WindowEvent { event: window_ev, .. } => {
                // egui_glium.egui_winit.
                let egui_consumed_event = egui_glium.on_event(&window_ev);

                if !egui_consumed_event {
                    if matches!(window_ev, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                        exit(control_flow, &shader_node_graph.graph.editor);
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
        // .window
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}