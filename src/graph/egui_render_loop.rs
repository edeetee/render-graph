use std::{path::{PathBuf, Path}, env, fs::File};

use egui_glium::EguiGlium;
use glium::glutin::{self, event::{Event, WindowEvent}, event_loop::ControlFlow};
use serde::{Serialize, de::DeserializeOwned};

use crate::graph::{def::EditorState, graph::ShaderGraph, self};

use super::ShaderGraphProcessor;

// const DEFAULT_FULLSCREEN_MODE: Option<Fullscreen> = Some(Fullscreen::Borderless(None));

// reference:
// https://github.com/emilk/egui/blob/master/egui_glium/examples/native_texture.rs

// const default_save_location = PathB

pub fn render_glium() {
    // let model = model::Model::new(options.num_stars, Some(options.model_options));
    // let model = ();
    let default_save_path = env::current_dir().unwrap().join("node_graph.json");

    let event_loop = glutin::event_loop::EventLoop::new();

    let display = create_display(&event_loop);

    println!("GL Vendor: {}", display.get_opengl_vendor_string());
    println!("GL Version: {}", display.get_opengl_version_string());

    let mut egui_glium = EguiGlium::new(&display);
    

    let mut shader_node_graph = match read_from_json_file(&default_save_path) {
        Ok(graph_state) => {
            println!("Loaded save file from {default_save_path:?}");
            ShaderGraphProcessor::new(ShaderGraph(graph_state, Default::default()))
        }
        Err(err) => {
            eprintln!("Failed to read default save {default_save_path:?} ({err:?}). Using new graph");
            ShaderGraphProcessor::default()
        },
    };
    

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
                    exit(control_flow, &shader_node_graph.graph.0);
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
                        exit(control_flow, &shader_node_graph.graph.0);
                    }
                }
            },
            _ => {}
        }
    });
}

fn write_to_json_file(path: &Path, data: &impl Serialize) -> anyhow::Result<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, data)?;

    Ok(())
}

fn read_from_json_file<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let file = File::open(path)?;
    Ok(serde_json::from_reader(file)?)
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