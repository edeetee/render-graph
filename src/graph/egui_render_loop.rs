use std::{path::{PathBuf, Path}, env, fs::File};

use egui_glium::EguiGlium;
use glium::glutin::{self, event::{Event, WindowEvent}, event_loop::ControlFlow};
use serde::{Serialize, de::DeserializeOwned};

use crate::graph::{graph::ShaderGraph, self};

use super::{ShaderGraphProcessor, def::{EditorState, ShaderNodeResponse}};

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

    let mut egui_glium = EguiGlium::new(&display, &event_loop);

    let mut shader_node_graph = match read_from_json_file::<EditorState>(&default_save_path) {
        Ok(graph_state) => {
            println!("Loaded save file from {default_save_path:?}");

            let new_nodes = graph_state.graph.nodes.iter()
                .map(|(node_id, ..)| egui_node_graph::NodeResponse::CreatedNode(node_id));

            let new_connections = graph_state.graph.connections.iter()
                .map(|(input, output)| egui_node_graph::NodeResponse::ConnectEventEnded{input, output: *output} );

            let events: Vec<ShaderNodeResponse> = new_nodes.chain(new_connections).collect();

            let mut shader_node_graph = ShaderGraphProcessor::new(ShaderGraph { editor: graph_state, tree: Default::default() });

            for event in events {
                shader_node_graph.node_event(&display, &mut egui_glium, event);
            }

            shader_node_graph
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